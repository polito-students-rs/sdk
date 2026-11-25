//! Browser login through the university single sign-on (SSO), as the
//! official app does it.
//!
//! The password goes only to the login page of the university, in a browser.
//! At the end the browser is redirected to `polito://students/login?uid=...&key=...`
//! and `uid` + `key` log in to the API ([`Credentials::sso`]). Forcing MFA
//! gives a secure session, which the MFA operations require (without one the
//! MFA status is `needsReauth`).

use reqwest::Url;

use super::{Auth, Credentials};
use crate::error::{ApiError, Result};

const START_URL: &str = "https://app.didattica.polito.it/auth/students/start";

/// One SSO login attempt, from [`Auth::sso_login`].
///
/// 1. open [`SsoLogin::url`] in a browser and log in there;
/// 2. the last page redirects to `polito://students/login?...`, a link meant
///    for the mobile app: a desktop browser cannot open it, copy it (e.g.
///    from the `Location` header of the last redirect in the developer tools);
/// 3. pass it to [`SsoLogin::credentials`], then the result to [`Auth::login`].
#[derive(Debug, Clone)]
pub struct SsoLogin {
    uid: String,
    url: Url,
}

impl SsoLogin {
    fn new(platform: &str, force_mfa: bool) -> Result<Self> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| ApiError::RandomUnavailable)?;
        let uid = uuid::Builder::from_random_bytes(bytes)
            .into_uuid()
            .hyphenated()
            .to_string();

        let mut url = Url::parse_with_params(START_URL, [("platform", platform), ("uid", &uid)])
            .map_err(|_| ApiError::InvalidBasePath(START_URL.to_owned()))?;
        if force_mfa {
            url
                .query_pairs_mut()
                .append_key_only("mfa");
        }
        Ok(Self { uid, url })
    }

    /// Page to open in a browser.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Identifier of this attempt, random: it ties the final redirect to it.
    pub fn uid(&self) -> &str {
        &self.uid
    }

    /// Credentials from the final redirect `polito://students/login?uid=...&key=...`.
    ///
    /// Fails with [`ApiError::InvalidSsoRedirect`], without details (the
    /// redirect carries the key), if it is not that link, has no key or
    /// belongs to another attempt.
    pub fn credentials(&self, redirect: &str) -> Result<Credentials> {
        let invalid = || ApiError::InvalidSsoRedirect;

        let url = Url::parse(redirect.trim()).map_err(|_| invalid())?;
        if url.scheme() != "polito" || url.host_str() != Some("students") || url.path() != "/login" {
            return Err(invalid());
        }

        let (mut uid, mut key) = (None, None);
        for (name, value) in url.query_pairs() {
            match name.as_ref() {
                "uid" => uid = Some(value),
                "key" => key = Some(value),
                _ => {}
            }
        }
        // the app keeps its own uid
        if uid.is_some_and(|uid| uid != self.uid) {
            return Err(invalid());
        }
        let key = key
            .filter(|key| !key.is_empty())
            .ok_or_else(invalid)?;
        Ok(Credentials::sso(self.uid.clone(), key.into_owned()))
    }
}

impl Auth<'_> {
    pub fn sso_login(&self, force_mfa: bool) -> Result<SsoLogin> {
        SsoLogin::new(&self.client.device.platform, force_mfa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn login(force_mfa: bool) -> SsoLogin {
        SsoLogin::new("android", force_mfa).unwrap()
    }

    fn redirect(query: &str) -> String {
        format!("polito://students/login?{query}")
    }

    #[test]
    fn url_opens_the_start_page() {
        let login = login(false);
        let url = login.url();
        assert_eq!(url.as_str().split('?').next(), Some(START_URL));
        let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert_eq!(
            pairs,
            [
                ("platform".to_owned(), "android".to_owned()),
                ("uid".to_owned(), login.uid().to_owned()),
            ]
        );
    }

    #[test]
    fn redirect_gives_sso_credentials() {
        let login = login(true);
        for query in [format!("uid={}&key=KEY", login.uid()), "key=KEY".to_owned()] {
            match login.credentials(&redirect(&query)).unwrap() {
                Credentials::Sso { uid, key } => {
                    assert_eq!(uid, login.uid());
                    assert_eq!(key, "KEY");
                }
                other => panic!("unexpected credentials: {other:?}"),
            }
        }
    }

    #[test]
    fn surrounding_whitespace_is_ignored() {
        let login = login(false);
        let pasted = format!("  {}\n", redirect("key=KEY"));
        assert!(login.credentials(&pasted).is_ok());
    }

    #[test]
    fn invalid_redirects_are_rejected() {
        let login = login(false);
        let other_attempt = redirect("uid=another&key=KEY");
        for link in [
            "not a url".to_owned(),
            "https://students/login?key=KEY".to_owned(),
            "polito://staff/login?key=KEY".to_owned(),
            "polito://students/logout?key=KEY".to_owned(),
            redirect(""),
            redirect("key="),
            other_attempt,
        ] {
            let result = login.credentials(&link);
            assert!(
                matches!(result, Err(ApiError::InvalidSsoRedirect)),
                "{link}: {result:?}"
            );
        }
    }

    #[test]
    fn errors_do_not_echo_the_key() {
        let login = login(false);
        let err = login
            .credentials(&redirect("uid=another&key=SECRET-KEY"))
            .unwrap_err();
        assert!(!format!("{err} {err:?}").contains("SECRET-KEY"));
    }
}
