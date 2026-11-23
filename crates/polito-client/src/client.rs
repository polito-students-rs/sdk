use std::sync::Arc;

use polito_api::apis::auth_api::{AuthApi, AuthApiClient};
use polito_api::apis::configuration::Configuration;
use polito_api::models::{Client as AppInfo, Device, update_preferences_request::Language};

use crate::auth::Auth;
use crate::error::{ApiError, Result};
use crate::middleware::AuthMiddleware;
use crate::session::{Session, SessionStore};

/// First server of the spec (production).
const DEFAULT_BASE_PATH: &str = polito_api::spec::SERVERS[0].url;
const DEFAULT_USER_AGENT: &str = concat!("polito-client/", env!("CARGO_PKG_VERSION"));

/// Entry point of the SDK.
///
/// Every generated API client shares one `Configuration` whose HTTP client
/// carries [`AuthMiddleware`]. Once logged in, the token is attached to every
/// request without rebuilding anything.
#[derive(Clone)]
pub struct PolitoClient {
    pub(crate) session: SessionStore,
    pub(crate) auth_api: Arc<dyn AuthApi>,
    pub(crate) app: Option<AppInfo>,
    pub(crate) device: Option<Device>,
    pub(crate) language: Option<Language>,
}

impl PolitoClient {
    pub fn builder() -> PolitoClientBuilder {
        PolitoClientBuilder::default()
    }

    pub fn auth(&self) -> Auth<'_> {
        Auth::new(self)
    }

    /// Current session, if logged in.
    pub fn session(&self) -> Option<Session> {
        self.session.get()
    }

    pub fn is_authenticated(&self) -> bool {
        self.session.token().is_some()
    }
}

#[derive(Default)]
pub struct PolitoClientBuilder {
    base_path: Option<String>,
    user_agent: Option<String>,
    http: Option<reqwest::Client>,
    app: Option<AppInfo>,
    device: Option<Device>,
    language: Option<Language>,
}

impl PolitoClientBuilder {
    /// Defaults to the production server.
    pub fn base_path(mut self, base_path: impl Into<String>) -> Self {
        self.base_path = Some(base_path.into());
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Custom `reqwest::Client` (timeouts, proxies, default headers...).
    /// The auth middleware is layered on top of it.
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    pub fn app_info(mut self, app: AppInfo) -> Self {
        self.app = Some(app);
        self
    }

    pub fn device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }

    /// Language preference sent on login.
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    pub fn build(self) -> Result<PolitoClient> {
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder().build()?,
        };

        let base_path = self
            .base_path
            .unwrap_or_else(|| DEFAULT_BASE_PATH.to_owned());
        let base_url = parse_base_path(&base_path)?;

        let session = SessionStore::default();
        let middleware_client = reqwest_middleware::ClientBuilder::new(http)
            .with(AuthMiddleware::new(session.clone(), &base_url))
            .build();

        let configuration = Arc::new(Configuration {
            base_path,
            user_agent: Some(
                self.user_agent
                    .unwrap_or_else(|| DEFAULT_USER_AGENT.to_owned()),
            ),
            client: middleware_client,
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        });

        Ok(PolitoClient {
            session,
            auth_api: Arc::new(AuthApiClient::new(configuration)),
            app: self.app,
            device: self.device,
            language: self.language,
        })
    }
}

/// The base path decides which origin receives the token.
fn parse_base_path(base_path: &str) -> Result<reqwest::Url> {
    let invalid = || ApiError::InvalidBasePath(base_path.to_owned());
    let url = reqwest::Url::parse(base_path).map_err(|_| invalid())?;
    match url.scheme() {
        "http" | "https" if url.has_host() => Ok(url),
        _ => Err(invalid()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_base_paths() {
        for base in [
            "https://app.didattica.polito.it/api",
            "http://127.0.0.1:4010",
        ] {
            assert!(
                PolitoClient::builder().base_path(base).build().is_ok(),
                "{base}"
            );
        }
    }

    #[test]
    fn rejects_invalid_base_paths() {
        for base in [
            "",
            "/api",
            "app.didattica.polito.it/api",
            "file:///tmp/api",
            "data:text/plain,x",
        ] {
            let result = PolitoClient::builder().base_path(base).build();
            assert!(
                matches!(result, Err(ApiError::InvalidBasePath(ref b)) if b == base),
                "{base}"
            );
        }
    }
}
