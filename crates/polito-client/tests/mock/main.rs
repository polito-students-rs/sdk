//! Integration tests against a Prism mock of `spec/openapi.yaml`.
//!
//! Custom runner (`harness = false`): starts the mock (or reuses one already
//! running, e.g. `just mock`), runs the tests, then stops it. Runs with plain
//! `cargo test`; `just test-mock` runs only this target. Usual libtest
//! arguments work, e.g. `cargo test --test mock -- announcements`.
//!
//! To add a test, write an `async fn` in a module and register it in `tests()`.

mod announcements;
mod config;
mod helpers;
mod server;

use std::future::Future;

use libtest_mimic::{Arguments, Trial};

fn tests() -> Vec<Trial> {
    vec![
        async_test("announcements::returns_spec_examples", announcements::returns_spec_examples),
        async_test("announcements::missing_token_is_unauthorized", announcements::missing_token_is_unauthorized),
        async_test("announcements::server_error_is_reported", announcements::server_error_is_reported),
        async_test("announcements::random_payloads_deserialize", announcements::random_payloads_deserialize),
    ]
}

fn main() {
    let args = Arguments::from_args();

    if args.list {
        libtest_mimic::run(&args, tests()).exit();
    }

    let server = match server::MockServer::start() {
        Ok(server) => server,
        Err(e) => {
            eprintln!("mock: {e:#}");
            std::process::exit(101);
        }
    };
    config::set_base_url(server.base_url());

    let conclusion = libtest_mimic::run(&args, tests());

    // `exit` skips destructors: stop Prism first
    drop(server);
    conclusion.exit();
}

fn async_test<F, Fut>(name: &'static str, test: F) -> Trial
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    Trial::test(name, move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(test());
        Ok(())
    })
}
