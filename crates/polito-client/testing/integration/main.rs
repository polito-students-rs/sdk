//! Integration tests against a Prism mock of `spec/openapi.yaml` with the
//! overlays applied (`target/codegen/openapi.yaml`).
//!
//! Custom runner (`harness = false`): starts the mock (or reuses one already
//! running, e.g. `just mock`), runs the tests, then stops it. Runs with plain
//! `cargo test`; `just test-integration` runs only this target. Usual libtest
//! arguments work, e.g. `cargo test --test integration -- announcements`.
//!
//! To add a test, write an `async fn` in a module and register it in `tests()`.

mod announcements;
mod auth;
mod config;
mod helpers;
mod mfa;
mod server;
mod service_link;

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
            "auth::login_with_custom_device",
            auth::login_with_custom_device,
        ),
        async_test(
            "auth::login_without_device_info_is_rejected",
            auth::login_without_device_info_is_rejected,
        ),
        async_test(
            "auth::random_login_payloads_deserialize",
            auth::random_login_payloads_deserialize,
        ),
        async_test("mfa::status_maps_spec_example", mfa::status_maps_spec_example),
        async_test(
            "mfa::pending_challenge_maps_spec_example",
            mfa::pending_challenge_maps_spec_example,
        ),
        async_test(
            "mfa::enrol_then_approve_and_decline",
            mfa::enrol_then_approve_and_decline,
        ),
        async_test(
            "mfa::restored_key_answers_only_its_challenges",
            mfa::restored_key_answers_only_its_challenges,
        ),
        async_test("mfa::mfa_requires_login", mfa::mfa_requires_login),
        async_test(
            "mfa::random_payloads_are_handled",
            mfa::random_payloads_are_handled,
        ),
        async_test("service_link::links_are_browser_urls", service_link::links_are_browser_urls),
        async_test("service_link::live_class_id_is_encoded", service_link::live_class_id_is_encoded),
        async_test("service_link::links_require_login", service_link::links_require_login),
        async_test("service_link::random_payloads_deserialize", service_link::random_payloads_deserialize),
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
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        rt.block_on(test());
        Ok(())
    })
}
