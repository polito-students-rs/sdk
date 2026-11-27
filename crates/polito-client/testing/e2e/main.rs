//! Manual end-to-end tests against the real API, with a real account.
//!
//! Never part of `cargo test` (`test = false`): run them with `just test-e2e`.
//! Usual libtest arguments work, e.g. `just test-e2e service_link`.
//!
//! The runner logs in once, runs the tests one at a time in registration
//! order and always logs out at the end.
//!
//! Environment variables, loaded from `testing/e2e/.env.test` (see the
//! `.env.test.example` next to it):
//! - `POLITO_E2E_USERNAME`, `POLITO_E2E_PASSWORD`: account, required;
//! - `POLITO_E2E_MOODLE_COURSE`: Moodle course id, needed by `moodle_link`;
//! - `POLITO_E2E_MEETING_ID`: LiveClass meeting id, needed by `live_class_link`.
//!
//! To add a test, write an `async fn(Setup) -> anyhow::Result<()>` in a
//! module and register it in `tests()`.

mod auth;
mod helpers;
mod mfa;
mod service_link;
mod setup;

use std::future::Future;

use anyhow::{Context, Result};
use libtest_mimic::{Arguments, Trial};
use polito_client::PolitoClient;
use polito_client::auth::Credentials;
use tokio::runtime::{Handle, Runtime};

use helpers::load_dotenv;
use setup::Setup;

/// Registration order is execution order: `auth::logout_invalidates_token`
/// must stay last.
/// `run` is what a trial needs to run: `None` when only listing.
fn tests(run: &Option<(Handle, Setup)>) -> Vec<Trial> {
    vec![
        async_test("auth::session_is_complete", auth::session_is_complete, run),
        async_test(
            "auth::no_session_is_unauthorized",
            auth::no_session_is_unauthorized,
            run,
        ),
        async_test("service_link::mail_link", service_link::mail_link, run),
        async_test(
            "service_link::smart_card_link",
            service_link::smart_card_link,
            run,
        ),
        async_test("service_link::moodle_link", service_link::moodle_link, run),
        async_test(
            "service_link::live_class_link",
            service_link::live_class_link,
            run,
        ),
        async_test("mfa::status_is_readable", mfa::status_is_readable, run),
        async_test(
            "mfa::status_details_follow_spec",
            mfa::status_details_follow_spec,
            run,
        ),
        async_test(
            "mfa::pending_challenge_is_readable",
            mfa::pending_challenge_is_readable,
            run,
        ),
        async_test(
            "auth::logout_invalidates_token",
            auth::logout_invalidates_token,
            run,
        ),
    ]
}

fn main() {
    let mut args = Arguments::from_args();
    args.test_threads = Some(1);

    if args.list {
        libtest_mimic::run(&args, tests(&None)).exit();
    }

    let runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("e2e: cannot start the tokio runtime: {e}");
            std::process::exit(101);
        }
    };
    let setup = match runtime.block_on(login()) {
        Ok(setup) => setup,
        Err(e) => {
            eprintln!("e2e: {e:#}");
            std::process::exit(101);
        }
    };

    let run = Some((runtime.handle().clone(), setup.clone()));
    let conclusion = libtest_mimic::run(&args, tests(&run));

    // `exit` skips destructors: no session must stay open on the server
    runtime.block_on(logout(&setup.client));
    conclusion.exit();
}

async fn login() -> Result<Setup> {
    load_dotenv()?;
    let base_url = polito_api::spec::SERVERS[0].url.to_owned();
    eprintln!("e2e: running against {base_url}");

    // secrets are never printed, not even in errors
    let credentials = Credentials::basic(
        std::env::var("POLITO_E2E_USERNAME").context("set POLITO_E2E_USERNAME in .env.test")?,
        std::env::var("POLITO_E2E_PASSWORD").context("set POLITO_E2E_PASSWORD in .env.test")?,
    );
    let client = PolitoClient::builder()
        .base_path(&base_url)
        .build()
        .context("cannot build the client")?;

    let session = client
        .auth()
        .login(credentials)
        .await
        .context("login failed")?;
    eprintln!(
        "e2e: logged in as {} ({})",
        session.username(),
        session.kind()
    );

    Ok(Setup { client, base_url })
}

/// Logs out if a test did not.
async fn logout(client: &PolitoClient) {
    if client.is_authenticated() {
        match client.auth().logout().await {
            Ok(()) => eprintln!("e2e: logged out"),
            Err(e) => eprintln!("e2e: logout failed: {e:?}"),
        }
    }
}

fn async_test<F, Fut>(name: &'static str, test: F, run: &Option<(Handle, Setup)>) -> Trial
where
    F: FnOnce(Setup) -> Fut + Send + 'static,
    Fut: Future<Output = Result<()>>,
{
    let run = run.clone();
    Trial::test(name, move || {
        let (handle, setup) = run.ok_or("e2e: not logged in")?;
        handle
            .block_on(test(setup))
            .map_err(|e| format!("{e:#}").into())
    })
}
