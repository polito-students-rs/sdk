//! Login/logout, MFA and service links.

mod login;
mod mfa;
mod sso;
mod service_link;

pub use login::Credentials;
pub use service_link::ServiceLink;
pub use mfa::{Challenge, Mfa, MfaDetails, MfaKey, MfaStatus};
pub use sso::SsoLogin;

use crate::client::PolitoClient;

/// Auth endpoints, obtained from [`PolitoClient::auth`].
pub struct Auth<'a> {
    client: &'a PolitoClient,
}

impl<'a> Auth<'a> {
    pub(crate) fn new(client: &'a PolitoClient) -> Self {
        Self { client }
    }
}
