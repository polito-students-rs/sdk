use polito_client::ApiError;
use polito_client::auth::Credentials;

use crate::config;

fn credentials() -> Credentials {
    Credentials::basic("s123456", "password")
}

pub async fn login_stores_session() {
    let sdk = config::sdk();
    assert!(!sdk.is_authenticated());

    let session = sdk.auth().login(credentials()).await.expect("login failed");

    assert!(sdk.is_authenticated());
    assert_eq!(sdk.session(), Some(session.clone()));
    assert!(!session.token().is_empty());
}

/// Login is anonymous in the spec: the active token must not be sent.
pub async fn login_again_while_logged_in() {
    let sdk = config::sdk();
    sdk.auth()
        .login(credentials())
        .await
        .expect("first login failed");

    sdk.auth()
        .login(credentials())
        .await
        .expect("second login failed");

    assert!(sdk.is_authenticated());
}

pub async fn sso_login_stores_session() {
    let sdk = config::sdk();

    sdk.auth()
        .login(Credentials::sso("uid", "key"))
        .await
        .expect("sso login failed");

    assert!(sdk.is_authenticated());
}

pub async fn protected_route_without_login_is_unauthorized() {
    let sdk = config::sdk();

    let err = sdk
        .auth()
        .switch_career("s123456")
        .await
        .expect_err("expected 401");

    assert!(
        matches!(err, ApiError::Unauthorized),
        "unexpected error: {err:?}"
    );
}

/// The middleware attaches the token: a protected route succeeds after login.
pub async fn token_is_sent_after_login() {
    let sdk = config::sdk();
    sdk.auth().login(credentials()).await.expect("login failed");

    let session = sdk
        .auth()
        .switch_career("s123456")
        .await
        .expect("switch career failed");

    assert_eq!(sdk.session(), Some(session));
}

pub async fn logout_clears_session() {
    let sdk = config::sdk();
    sdk.auth().login(credentials()).await.expect("login failed");

    sdk.auth().logout().await.expect("logout failed");

    assert!(!sdk.is_authenticated());
    let err = sdk
        .auth()
        .switch_career("s123456")
        .await
        .expect_err("expected 401");
    assert!(
        matches!(err, ApiError::Unauthorized),
        "unexpected error: {err:?}"
    );
}

pub async fn logout_without_session_is_noop() {
    let sdk = config::sdk();

    sdk.auth().logout().await.expect("logout failed");

    assert!(!sdk.is_authenticated());
}

pub async fn login_server_error_is_reported() {
    let sdk = config::sdk_with_prefer("code=500");

    let err = sdk
        .auth()
        .login(credentials())
        .await
        .expect_err("expected 500");

    assert!(
        matches!(err, ApiError::Server { status: 500, .. }),
        "unexpected error: {err:?}"
    );
    assert!(!sdk.is_authenticated());
}

pub async fn login_bad_request_is_rejected() {
    let sdk = config::sdk_with_prefer("code=400");

    let err = sdk
        .auth()
        .login(credentials())
        .await
        .expect_err("expected 400");

    assert!(
        matches!(err, ApiError::Rejected { status: 400, .. }),
        "unexpected error: {err:?}"
    );
}

pub async fn random_login_payloads_deserialize() {
    let sdk = config::sdk_with_prefer("dynamic=true");

    for _ in 0..10 {
        sdk.auth()
            .login(credentials())
            .await
            .expect("dynamic payload did not deserialize");
    }
}
