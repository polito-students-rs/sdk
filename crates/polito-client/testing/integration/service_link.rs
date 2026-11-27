use polito_client::auth::{Credentials, ServiceLink};
use polito_client::{ApiError, PolitoClient};

use crate::config;

async fn logged_in(sdk: PolitoClient) -> PolitoClient {
    sdk.auth()
        .login(Credentials::basic("s123456", "password"))
        .await
        .expect("login failed");
    sdk
}

fn assert_browser_url(link: &ServiceLink) {
    let url = link.url();
    assert!(
        matches!(url.scheme(), "http" | "https"),
        "scheme {}",
        url.scheme()
    );
    assert!(url.has_host(), "missing host");
}

pub async fn links_are_browser_urls() {
    let sdk = logged_in(config::sdk()).await;
    let auth = sdk.auth();

    let links = [
        auth.mail_link().await.expect("mail link"),
        auth.moodle_link(12345).await.expect("moodle link"),
        auth.live_class_link("meeting-1")
            .await
            .expect("live class link"),
        auth.smart_card_link().await.expect("smart card link"),
    ];

    links.iter().for_each(assert_browser_url);
}

/// The meeting id is a path segment: reserved characters must be encoded,
/// not change the route.
pub async fn live_class_id_is_encoded() {
    let sdk = logged_in(config::sdk()).await;

    let link = sdk
        .auth()
        .live_class_link("a/b c?d")
        .await
        .expect("live class link");

    assert_browser_url(&link);
}

pub async fn links_require_login() {
    let sdk = config::sdk();

    let err = sdk.auth().mail_link().await.expect_err("expected 401");

    assert!(
        matches!(err, ApiError::Unauthorized),
        "unexpected error: {err:?}"
    );
}

/// Random payloads (extra fields, random URIs) must deserialize.
pub async fn random_payloads_deserialize() {
    let sdk = logged_in(config::sdk_with_prefer("dynamic=true")).await;
    let auth = sdk.auth();

    for _ in 0..10 {
        check_random(auth.mail_link().await, "mail");
        check_random(auth.moodle_link(1).await, "moodle");
        check_random(auth.live_class_link("m").await, "live class");
        check_random(auth.smart_card_link().await, "smart card");
    }
}

fn check_random(result: polito_client::Result<ServiceLink>, name: &str) {
    match result {
        Ok(link) => assert_browser_url(&link),
        Err(ApiError::Malformed(msg)) if msg.starts_with("service link") => {}
        Err(e) => panic!("{name} link: {e:?}"),
    }
}
