//! MFA against the mock.

use chrono::{DateTime, Utc};
use polito_client::auth::{Credentials, MfaKey, MfaStatus};
use polito_client::{ApiError, PolitoClient};

use crate::config;

const SERIAL: &str = "EDUP000123456";

async fn logged_in(sdk: PolitoClient) -> PolitoClient {
    sdk.auth()
        .login(Credentials::basic("s123456", "password"))
        .await
        .expect("login failed");
    sdk
}

fn utc(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("valid date")
        .with_timezone(&Utc)
}

pub async fn status_maps_spec_example() {
    let sdk = logged_in(config::sdk()).await;

    let status = sdk.auth().mfa().status().await.expect("status");

    let MfaStatus::Active(details) = status else {
        panic!("expected Active, got {status:?}");
    };
    assert_eq!(details.last_auth(), utc("2021-09-30T00:00:00Z"));
    assert_eq!(details.auth_count(), 1);
    assert_eq!(details.serial(), Some("EDUP0000123456"));
    assert_eq!(details.description(), Some("My phone"));
}

pub async fn pending_challenge_maps_spec_example() {
    let sdk = logged_in(config::sdk()).await;

    let challenge = sdk
        .auth()
        .mfa()
        .pending_challenge()
        .await
        .expect("challenge")
        .expect("no pending challenge");

    assert_eq!(challenge.serial(), SERIAL);
    assert_eq!(challenge.requested_at(), utc("2025-01-01T00:00:00Z"));
    assert_eq!(challenge.expires_at(), utc("2025-01-01T01:00:00Z"));
    assert!(!challenge.is_expired(utc("2025-01-01T00:30:00Z")));
    assert!(challenge.is_expired(utc("2025-01-01T01:00:00Z")));
}

/// The whole flow: enrol, fetch the pending login request, answer it.
pub async fn enrol_then_approve_and_decline() {
    let sdk = logged_in(config::sdk()).await;
    let mfa = sdk.auth().mfa();

    let key = mfa.enrol("sdk integration test").await.expect("enrol");
    assert_eq!(key.serial(), SERIAL);

    let challenge = mfa
        .pending_challenge()
        .await
        .expect("challenge")
        .expect("no pending challenge");
    mfa.approve(&challenge, &key).await.expect("approve");
    mfa.decline(&challenge, &key).await.expect("decline");
}

/// A key saved with `to_parts` answers after `from_parts`; restored with
/// another serial it is refused before any request.
pub async fn restored_key_answers_only_its_challenges() {
    let sdk = logged_in(config::sdk()).await;
    let mfa = sdk.auth().mfa();
    let (serial, secret) = mfa.enrol("sdk integration test").await.expect("enrol").to_parts();
    let challenge = mfa
        .pending_challenge()
        .await
        .expect("challenge")
        .expect("no pending challenge");

    let restored = MfaKey::from_parts(serial, &secret).expect("restore");
    mfa.approve(&challenge, &restored).await.expect("approve");

    let other = MfaKey::from_parts("OTHER".to_owned(), &secret).expect("restore");
    for result in [
        mfa.approve(&challenge, &other).await,
        mfa.decline(&challenge, &other).await,
    ] {
        assert!(
            matches!(result, Err(ApiError::MfaDifferentSerial)),
            "unexpected result: {result:?}"
        );
    }
}

pub async fn mfa_requires_login() {
    let sdk = config::sdk();

    let err = sdk.auth().mfa().status().await.expect_err("expected 401");

    assert!(
        matches!(err, ApiError::Unauthorized),
        "unexpected error: {err:?}"
    );
}

/// Random payloads: each shape allowed by the spec either maps to the SDK
/// types or, when the SDK cannot represent it (`active`/`locked` without
/// details, negative counts), is reported as `Malformed`. `success: false`
/// from validate becomes `MfaValidationFailed`.
pub async fn random_payloads_are_handled() {
    let sdk = logged_in(config::sdk_with_prefer("dynamic=true")).await;
    let mfa = sdk.auth().mfa();
    let (_, secret) = mfa.enrol("sdk integration test").await.expect("enrol").to_parts();

    for _ in 0..10 {
        match mfa.status().await {
            Ok(_) => {}
            Err(ApiError::Malformed(msg))
                if msg.starts_with("MFA status") || msg.starts_with("negative MFA auth count") => {}
            Err(e) => panic!("status: {e:?}"),
        }

        let Some(challenge) = mfa.pending_challenge().await.expect("challenge") else {
            continue;
        };
        // random serial: answer with a key restored for it
        let key = MfaKey::from_parts(challenge.serial().to_owned(), &secret).expect("restore");
        for result in [
            mfa.approve(&challenge, &key).await,
            mfa.decline(&challenge, &key).await,
        ] {
            match result {
                Ok(()) | Err(ApiError::MfaValidationFailed) => {}
                Err(e) => panic!("validate: {e:?}"),
            }
        }
    }
}
