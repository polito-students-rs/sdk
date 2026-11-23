use std::fmt;

use polito_api::models::{
    LoginCredentialsBasic,
    LoginCredentialsSso,
    LoginRequest,
    SwitchCareerRequest,
    UpdatePreferencesRequest,
    login_credentials_basic,
    login_credentials_sso,
};

use super::Auth;
use crate::error::{ApiError, Result};
use crate::session::Session;

/// Login credentials.
#[derive(Clone)]
pub enum Credentials {
    Basic { username: String, password: String },
    Sso { uid: String, key: String },
}

impl Credentials {
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn sso(uid: impl Into<String>, key: impl Into<String>) -> Self {
        Self::Sso {
            uid: uid.into(),
            key: key.into(),
        }
    }
}

// secrets never end up in logs
impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic { username, .. } => f
                .debug_struct("Basic")
                .field("username", username)
                .field("password", &"<redacted>")
                .finish(),
            Self::Sso { uid, .. } => f
                .debug_struct("Sso")
                .field("uid", uid)
                .field("key", &"<redacted>")
                .finish(),
        }
    }
}

impl Auth<'_> {
    /// Logs in and stores the session.
    pub async fn login(&self, credentials: Credentials) -> Result<Session> {
        let request = self.login_request(credentials);
        let response = self.client.auth_api.auth_login(request).await?;
        Ok(self.store(*response.data))
    }

    /// Invalidates the token server side and drops the local session.
    ///
    /// The local session is dropped even when the request fails. An already
    /// expired token is not an error.
    pub async fn logout(&self) -> Result<()> {
        if !self.client.is_authenticated() {
            return Ok(());
        }
        let result = self.client.auth_api.auth_logout().await;
        self.client.session.clear();
        match result.map_err(ApiError::from) {
            Ok(()) | Err(ApiError::Unauthorized) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Switches to another career of the same user. The new token replaces
    /// the current one.
    pub async fn switch_career(&self, username: impl Into<String>) -> Result<Session> {
        let request = SwitchCareerRequest {
            username: username.into(),
            client: self.client.app.clone().map(Box::new),
            device: self.client.device.clone().map(Box::new),
        };
        let response = self.client.auth_api.auth_switch_career(request).await?;
        Ok(self.store(*response.data))
    }

    fn store(&self, identity: polito_api::models::Identity) -> Session {
        let session = Session::from_identity(identity);
        self.client.session.set(session.clone());
        session
    }

    fn login_request(&self, credentials: Credentials) -> LoginRequest {
        let client = self.client.app.clone().map(Box::new);
        let device = self.client.device.clone().map(Box::new);
        let preferences = Box::new(UpdatePreferencesRequest {
            language: self.client.language,
        });

        match credentials {
            Credentials::Basic { username, password } => {
                LoginRequest::Basic(Box::new(LoginCredentialsBasic {
                    client,
                    device,
                    preferences,
                    username,
                    password,
                    login_type: login_credentials_basic::LoginType::Basic,
                }))
            }
            Credentials::Sso { uid, key } => LoginRequest::Sso(Box::new(LoginCredentialsSso {
                client,
                device,
                preferences,
                uid,
                key,
                login_type: login_credentials_sso::LoginType::Sso,
            })),
        }
    }
}
