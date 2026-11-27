use anyhow::{Context, Result, ensure};
use polito_client::auth::ServiceLink;

use crate::setup::Setup;

fn check(name: &str, link: &ServiceLink) -> Result<()> {
    let url = link.url();
    ensure!(url.scheme() == "https", "{name}: scheme {:?}", url.scheme());
    eprintln!("    {name} -> {}", url.host_str().unwrap_or_default());
    Ok(())
}

pub async fn mail_link(setup: Setup) -> Result<()> {
    let link = setup.client.auth().mail_link().await?;
    check("mail", &link)
}

pub async fn smart_card_link(setup: Setup) -> Result<()> {
    let link = setup.client.auth().smart_card_link().await?;
    check("smart card", &link)
}

pub async fn moodle_link(setup: Setup) -> Result<()> {
    let var = setup.require("POLITO_E2E_MOODLE_COURSE")?;
    let course = var
        .parse()
        .with_context(|| format!("invalid POLITO_E2E_MOODLE_COURSE {var:?}"))?;
    let link = setup.client.auth().moodle_link(course).await?;
    check("moodle", &link)
}

pub async fn live_class_link(setup: Setup) -> Result<()> {
    let meeting = setup.require("POLITO_E2E_MEETING_ID")?;
    let link = setup.client.auth().live_class_link(&meeting).await?;
    check("live class", &link)
}
