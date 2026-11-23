//! Integration tests against a Prism mock of `spec/openapi.yaml`.
//!
//! Custom runner (`harness = false`): starts the mock (or reuses one already
//! running, e.g. `just mock`), runs the tests, then stops it. Runs with plain
//! `cargo test`; `just test-mock` runs only this target. Usual libtest
//! arguments work, e.g. `cargo test --test mock -- announcements`.
//!
//! To add a test, write an `async fn` in a module and register it in `tests()`.

mod announcements;
mod auth;
mod config;
mod helpers;
mod server;

use std::future::Future;

use libtest_mimic::{Arguments, Trial};

fn tests() -> Vec<Trial> {
    vec![
        async_test(
            "announcements::returns_spec_examples",
            announcements::returns_spec_examples,
        ),
        async_test(
            "announcements::missing_token_is_unauthorized",
            announcements::missing_token_is_unauthorized,
        ),
        async_test(
            "announcements::server_error_is_reported",
            announcements::server_error_is_reported,
        ),
        async_test(
            "announcements::random_payloads_deserialize",
            announcements::random_payloads_deserialize,
        ),
        async_test("auth::login_stores_session", auth::login_stores_session),
        async_test(
            "auth::login_again_while_logged_in",
            auth::login_again_while_logged_in,
        ),
        async_test(
            "auth::sso_login_stores_session",
            auth::sso_login_stores_session,
        ),
        async_test(
            "auth::protected_route_without_login_is_unauthorized",
            auth::protected_route_without_login_is_unauthorized,
        ),
        async_test(
            "auth::token_is_sent_after_login",
            auth::token_is_sent_after_login,
        ),
        async_test("auth::logout_clears_session", auth::logout_clears_session),
        async_test(
            "auth::logout_without_session_is_noop",
            auth::logout_without_session_is_noop,
        ),
        async_test(
            "auth::login_server_error_is_reported",
            auth::login_server_error_is_reported,
        ),
        async_test(
            "auth::login_bad_request_is_rejected",
            auth::login_bad_request_is_rejected,
        ),
        async_test(
            "auth::random_login_payloads_deserialize",
            auth::random_login_payloads_deserialize,
        ),
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
