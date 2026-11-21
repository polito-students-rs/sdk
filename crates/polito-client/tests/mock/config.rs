//! `Configuration`s pointing at the mock server.
//!
//! Prism serves the paths at the root, without the `/api` prefix of the
//! production `servers.url`, and accepts any bearer token.

#![allow(dead_code)]

use std::sync::{Arc, OnceLock};

use polito_api::apis::configuration::Configuration;
use reqwest::header::{HeaderMap, HeaderValue};

static BASE_URL: OnceLock<String> = OnceLock::new();

pub fn set_base_url(url: &str) {
    BASE_URL.set(url.to_owned()).expect("base url set twice");
}

fn base_url() -> String {
    BASE_URL.get().expect("mock server not started").clone()
}

/// Authenticated configuration. Prism only checks that the header is present.
pub fn authenticated() -> Arc<Configuration> {
    Arc::new(Configuration {
        base_path: base_url(),
        bearer_access_token: Some("mock-token".to_owned()),
        ..Default::default()
    })
}

/// Configuration without a token, so protected routes answer 401.
pub fn unauthenticated() -> Arc<Configuration> {
    Arc::new(Configuration {
        base_path: base_url(),
        ..Default::default()
    })
}

/// Authenticated configuration that sends `Prefer: <prefer>` on every request,
/// e.g. `code=500` (only codes declared in the spec) or `dynamic=true`.
pub fn with_prefer(prefer: &str) -> Arc<Configuration> {
    let mut headers = HeaderMap::new();
    headers.insert("Prefer", HeaderValue::from_str(prefer).expect("valid header value"));
    let http = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("reqwest client");

    Arc::new(Configuration {
        base_path: base_url(),
        bearer_access_token: Some("mock-token".to_owned()),
        client: reqwest_middleware::ClientBuilder::new(http).build(),
        ..Default::default()
    })
}
