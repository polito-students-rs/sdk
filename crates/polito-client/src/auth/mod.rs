//! Login/logout, MFA and service links.

mod login;
mod mfa;
mod service_link;

pub use login::Credentials;

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
