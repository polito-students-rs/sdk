use anyhow::{Context, Result, ensure};
use polito_client::{ApiError, PolitoClient};

use crate::setup::Setup;

pub async fn session_is_complete(setup: Setup) -> Result<()> {
    let session = setup.client.session().context("not logged in")?;

    ensure!(!session.username().is_empty(), "empty username");
    ensure!(!session.kind().is_empty(), "empty identity type");
    ensure!(!session.client_id().is_empty(), "empty client id");
    ensure!(!session.token().is_empty(), "empty token");
    Ok(())
}

/// A protected route without token answers 401 (and not 403, 400...).
pub async fn no_session_is_unauthorized(setup: Setup) -> Result<()> {
    let anonymous = PolitoClient::builder().base_path(&setup.base_url).build()?;

    match anonymous.auth().mail_link().await {
        Err(ApiError::Unauthorized) => Ok(()),
        Err(e) => anyhow::bail!("expected Unauthorized, got {e:?}"),
        Ok(_) => anyhow::bail!("protected route answered without a token"),
    }
}

/// Logout revokes the token server side, not only locally. Must run last.
pub async fn logout_invalidates_token(setup: Setup) -> Result<()> {
    let token = setup
        .client
        .session()
        .context("not logged in")?
        .token()
        .to_owned();

    setup
        .client
        .auth()
        .logout()
        .await
        .context("logout failed")?;
    ensure!(
        !setup.client.is_authenticated(),
        "local session still present"
    );

    // the old token, sent by hand, must now be rejected
    let status = reqwest::Client::new()
        .get(format!("{}/auth/serviceLink/mail", setup.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .context("request with the revoked token failed")?
        .status();
    ensure!(status == 401, "revoked token answered {status}");
    Ok(())
}
