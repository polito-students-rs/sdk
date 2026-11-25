//! MFA, read-only.
//!
//! `enrol` may replace the authenticator in use (the PoliTO app on the phone)
//! and a wrong answer to a real challenge may count as a failed attempt and
//! lock it: `enrol`, `approve` and `decline` are never run here.

use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use chrono::{TimeDelta, Utc};
use polito_api::apis::auth_api::{AuthApi, AuthApiClient};
use polito_api::apis::configuration::Configuration;
use polito_api::models::mfa_status_response::Status;
use polito_client::ApiError;
use polito_client::auth::MfaStatus;

use crate::setup::Setup;

const CLOCK_SKEW: TimeDelta = TimeDelta::minutes(5);

fn state(status: &MfaStatus) -> &'static str {
    match status {
        MfaStatus::Active(_) => "active",
        MfaStatus::Locked(_) => "locked",
        MfaStatus::Available => "available",
        MfaStatus::Unavailable => "unavailable",
        MfaStatus::NeedsReauth => "needsReauth",
        _ => "unknown",
    }
}

pub async fn status_is_readable(setup: Setup) -> Result<()> {
    let status = setup.client.auth().mfa().status().await.context("status")?;

    if let MfaStatus::Active(details) | MfaStatus::Locked(details) = &status {
        ensure!(
            details.last_auth() <= Utc::now() + CLOCK_SKEW,
            "last authentication in the future: {}",
            details.last_auth()
        );
        ensure!(
            details.serial().is_none_or(|serial| !serial.is_empty()),
            "empty serial"
        );
        eprintln!(
            "    status: {} ({} authentications)",
            state(&status),
            details.auth_count()
        );
    } else {
        eprintln!("    status: {}", state(&status));
    }
    Ok(())
}

pub async fn status_details_follow_spec(setup: Setup) -> Result<()> {
    let token = setup
        .client
        .session()
        .context("not logged in")?
        .token()
        .to_owned();
    let api = AuthApiClient::new(Arc::new(Configuration {
        base_path: setup.base_url.clone(),
        user_agent: Some(concat!("polito-client/", env!("CARGO_PKG_VERSION")).to_owned()),
        bearer_access_token: Some(token),
        ..Default::default()
    }));

    let data = api.mfa_get_mfa_status().await.context("status")?.data;

    let expected = matches!(data.status, Status::Active | Status::Locked);
    ensure!(
        data.details.is_some() == expected,
        "status {:?} {} details: the spec has them only for active and locked",
        data.status,
        if data.details.is_some() { "with" } else { "without" },
    );
    Ok(())
}

/// MFA operations need a secure session (SSO login with MFA, as the official
/// app does): with `needsReauth`, e.g. after the password login of this
/// runner, the server refuses the challenge with 401.
pub async fn pending_challenge_is_readable(setup: Setup) -> Result<()> {
    let mfa = setup.client.auth().mfa();
    let status = mfa.status().await.context("status")?;

    let challenge = match mfa.pending_challenge().await {
        Ok(challenge) => challenge,
        Err(ApiError::Unauthorized) if matches!(status, MfaStatus::NeedsReauth) => {
            eprintln!("    needsReauth: challenge refused with 401, as expected");
            return Ok(());
        }
        Err(e) => return Err(e).context("pending challenge"),
    };

    let Some(challenge) = challenge else {
        eprintln!("    no pending challenge");
        return Ok(());
    };
    ensure!(!challenge.serial().is_empty(), "empty serial");
    ensure!(
        challenge.requested_at() < challenge.expires_at(),
        "expires at {} but was requested at {}",
        challenge.expires_at(),
        challenge.requested_at()
    );
    eprintln!(
        "    pending challenge, {}s left",
        (challenge.expires_at() - Utc::now()).num_seconds()
    );
    Ok(())
}


