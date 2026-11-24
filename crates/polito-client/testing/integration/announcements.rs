use polito_api::apis::{
    Error,
    announcements_api::{AnnouncementsApi, AnnouncementsApiClient},
};

use crate::config;

pub async fn returns_spec_examples() {
    let api = AnnouncementsApiClient::new(config::authenticated());

    let res = api
        .announcements_get_announcements(None)
        .await
        .expect("request failed");

    // static examples from the spec: deterministic
    assert_eq!(res.data.len(), 2);
    assert_eq!(res.data[0].id, "abc-123");
    assert!(!res.data[0].seen);
}

pub async fn missing_token_is_unauthorized() {
    let api = AnnouncementsApiClient::new(config::unauthenticated());

    let err = api
        .announcements_get_announcements(None)
        .await
        .expect_err("expected 401");

    match err {
        Error::ResponseError(r) => assert_eq!(r.status, 401),
        other => panic!("unexpected error: {other}"),
    }
}

pub async fn server_error_is_reported() {
    let api = AnnouncementsApiClient::new(config::with_prefer("code=500"));

    let err = api
        .announcements_get_announcements(None)
        .await
        .expect_err("expected 500");

    match err {
        Error::ResponseError(r) => assert_eq!(r.status, 500),
        other => panic!("unexpected error: {other}"),
    }
}

pub async fn random_payloads_deserialize() {
    let api = AnnouncementsApiClient::new(config::with_prefer("dynamic=true"));

    // checks that models accept every valid shape
    for _ in 0..10 {
        api.announcements_get_announcements(None)
            .await
            .expect("dynamic payload did not deserialize");
    }
}
