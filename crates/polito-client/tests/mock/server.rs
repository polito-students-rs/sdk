//! Lifecycle of the Prism mock server used by the integration tests.
//!
//! Address comes from `MOCK_HOST` / `MOCK_PORT` (loaded from `.env.test` and
//! `.env` at the workspace root, defaults `127.0.0.1:4010`). If something is
//! already listening at that address (e.g. `just mock`),
//! it is reused; otherwise Prism is started from the local `node_modules` and
//! stopped when the guard is dropped.

use std::fs::File;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail, ensure};

use crate::helpers::{load_dotenv, workspace_root};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

pub struct MockServer {
    base_url: String,
    child: Option<Child>,
}

impl MockServer {
    pub fn start() -> Result<Self> {
        let root = workspace_root()?;
        load_dotenv()?;

        let ip = std::env::var("MOCK_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = match std::env::var("MOCK_PORT") {
            Ok(p) => p.parse::<u16>().with_context(|| format!("invalid MOCK_PORT {p:?}"))?,
            Err(_) => 4010,
        };
        // a server bound to every interface is reached through loopback
        let host = if ip == "0.0.0.0" { "127.0.0.1" } else { ip.as_str() };
        let base_url = format!("http://{host}:{port}");
        let addr = resolve(host, port)?;

        if is_listening(&addr) {
            eprintln!("mock: reusing server already running at {base_url}");
            return Ok(Self { base_url, child: None });
        }

        let prism = root.join("node_modules/@stoplight/prism-cli/dist/index.js");
        ensure!(
            prism.exists(),
            "Prism not found at {}: run `npm install` in the workspace root",
            prism.display()
        );

        let log_path = root.join("target/prism-mock.log");
        let log = File::create(&log_path)
            .with_context(|| format!("cannot create {}", log_path.display()))?;
        let log_err = log.try_clone().context("cannot clone Prism log handle")?;

        eprintln!("mock: starting Prism at {base_url} (log: {})", log_path.display());
        let child = Command::new("node")
            .arg(&prism)
            .args(["mock", "--errors", "-h", &ip, "-p", &port.to_string(), "spec/openapi.yaml"])
            .current_dir(&root)
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(log_err)
            .spawn()
            .context("cannot start Prism (is `node` on PATH?)")?;

        // from here on the guard kills Prism even if startup fails
        let mut server = Self { base_url, child: Some(child) };
        server.wait_until_ready(&addr, &log_path)?;
        Ok(server)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn wait_until_ready(&mut self, addr: &SocketAddr, log_path: &Path) -> Result<()> {
        let deadline = Instant::now() + STARTUP_TIMEOUT;
        while Instant::now() < deadline {
            if is_listening(addr) {
                return Ok(());
            }
            if let Some(child) = self.child.as_mut()
                && let Ok(Some(status)) = child.try_wait()
            {
                bail!("Prism exited with {status}, see {}", log_path.display());
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        bail!(
            "Prism not ready after {}s, see {}",
            STARTUP_TIMEOUT.as_secs(),
            log_path.display()
        )
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn resolve(host: &str, port: u16) -> Result<SocketAddr> {
    (host, port)
        .to_socket_addrs()
        .with_context(|| format!("invalid mock address {host}:{port}"))?
        .next()
        .with_context(|| format!("cannot resolve {host}:{port}"))
}

fn is_listening(addr: &SocketAddr) -> bool {
    TcpStream::connect_timeout(addr, Duration::from_millis(200)).is_ok()
}
