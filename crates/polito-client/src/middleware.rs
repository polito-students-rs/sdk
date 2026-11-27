//! Injects the session token into every request.
//!
//! The generated `Configuration` keeps `bearer_access_token: None`: the token
//! lives in the [`SessionStore`] and changes there (login, logout, switch
//! career) without rebuilding configuration or API clients.
//!
//! The token is attached only to requests whose origin (scheme, host, port)
//! matches the API `base_path`: a URL to a third party (service links, files
//! hosted elsewhere) fetched with the same client never receives it.

use http::Extensions;
use polito_api::spec::{OPERATIONS, Operation};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use reqwest::{Method, Request, Response, Url};
use reqwest_middleware::{Middleware, Next};
use url::Origin;

use crate::session::SessionStore;

fn find_operation(method: &Method, path: &str) -> Option<&'static Operation> {
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    OPERATIONS
        .iter()
        .filter(|op| op.method == method.as_str())
        .filter_map(|op| literal_matches(op.path, &segments).map(|literals| (op, literals)))
        .max_by_key(|(_, literals)| *literals)
        .map(|(op, _)| op)
}

/// Number of literal segments of `template` if it matches `segments`.
fn literal_matches(template: &str, segments: &[&str]) -> Option<usize> {
    let template: Vec<&str> = template.trim_matches('/').split('/').collect();
    if template.len() != segments.len() {
        return None;
    }
    let mut literals = 0;
    for (expected, actual) in template.iter().zip(segments) {
        if is_template(expected) {
            if actual.is_empty() {
                return None;
            }
        } else if expected == actual {
            literals += 1;
        } else {
            return None;
        }
    }
    Some(literals)
}

fn is_template(segment: &str) -> bool {
    segment.starts_with('{') && segment.ends_with('}')
}

#[derive(Debug, thiserror::Error)]
#[error("session token is not a valid header value")]
struct InvalidToken;

pub(crate) struct AuthMiddleware {
    store: SessionStore,
    origin: Origin,
    /// Path of the base URL without trailing slash (e.g. `/api`): the spec
    /// paths are relative to it.
    base_path: String,
}

impl AuthMiddleware {
    pub(crate) fn new(store: SessionStore, base_url: &Url) -> Self {
        Self {
            store,
            origin: base_url.origin(),
            base_path: base_url.path().trim_end_matches('/').to_owned(),
        }
    }

    fn is_api_origin(&self, req: &Request) -> bool {
        req.url().origin() == self.origin
    }

    /// The spec declares the operation callable without credentials
    /// (`security: [{}]` or `security: []`), e.g. login, it must not carry a
    /// token. Paths outside the base path or unknown to the spec are not
    /// anonymous.
    fn is_anonymous(&self, req: &Request) -> bool {
        req.url()
            .path()
            .strip_prefix(self.base_path.as_str())
            .filter(|path| path.is_empty() || path.starts_with('/'))
            .and_then(|path| find_operation(req.method(), path))
            .is_some_and(Operation::allows_anonymous)
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        // an explicit header set by the caller wins
        if !req.headers().contains_key(AUTHORIZATION)
            && self.is_api_origin(&req)
            && !self.is_anonymous(&req)
            && let Some(token) = self.store.token()
        {
            let mut value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| reqwest_middleware::Error::middleware(InvalidToken))?;
            value.set_sensitive(true);
            req.headers_mut().insert(AUTHORIZATION, value);
        }
        next.run(req, extensions).await
    }
}
