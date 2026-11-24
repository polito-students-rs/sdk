//! Authenticated links to external services (mail, Moodle, live class,
//! smart card).
//!
//! The API returns a URL already authorised for the current user, to open in
//! a browser without a second login. The URL may embed a short-lived
//! credential, so [`ServiceLink`] never prints it in full.

use std::fmt;

use polito_api::models::ServiceLinkGetLiveClassLink200Response as LinkResponse;
use reqwest::Url;

use super::Auth;
use crate::error::{ApiError, Result};

/// URL to an external service, authorised for the current user.
///
/// `Debug` shows only scheme, host and path: query, fragment and user info
/// may carry a credential.
#[derive(Clone, PartialEq, Eq)]
pub struct ServiceLink(Url);

impl ServiceLink {
    fn parse(response: LinkResponse) -> Result<Self> {
        let url = Url::parse(&response.data.url)
            .map_err(|e| ApiError::Malformed(format!("service link is not a valid URL: {e}")))?;
        match url.scheme() {
            "http" | "https" if url.has_host() => Ok(Self(url)),
            scheme => Err(ApiError::Malformed(format!(
                "service link has unsupported scheme {scheme:?}"
            ))),
        }
    }

    pub fn url(&self) -> &Url {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

impl fmt::Debug for ServiceLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let url = &self.0;
        f.debug_tuple("ServiceLink")
            .field(&format_args!(
                "{}://{}{}{}",
                url.scheme(),
                url.host_str().unwrap_or_default(),
                url.port().map(|p| format!(":{p}")).unwrap_or_default(),
                url.path(),
            ))
            .finish()
    }
}

impl Auth<'_> {
    /// Webmail of the university.
    pub async fn mail_link(&self) -> Result<ServiceLink> {
        let response = self.client.auth_api.service_link_get_mail_link().await?;
        ServiceLink::parse(response)
    }

    /// Moodle course, by Moodle course identifier.
    pub async fn moodle_link(&self, course: i32) -> Result<ServiceLink> {
        let response = self
            .client
            .auth_api
            .service_link_get_moodle_link(course)
            .await?;
        ServiceLink::parse(response)
    }

    /// LiveClass meeting (virtual classroom), by meeting identifier.
    pub async fn live_class_link(&self, meeting_id: &str) -> Result<ServiceLink> {
        let response = self
            .client
            .auth_api
            .service_link_get_live_class_link(meeting_id)
            .await?;
        ServiceLink::parse(response)
    }

    /// Authenticated endpoint of the smart card picture.
    pub async fn smart_card_link(&self) -> Result<ServiceLink> {
        let response = self
            .client
            .auth_api
            .service_link_get_smart_card_link()
            .await?;
        ServiceLink::parse(response)
    }
}
