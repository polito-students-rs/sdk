//! What every e2e test receives.

use anyhow::{Context, Result};
use polito_client::PolitoClient;

/// Inputs of a test, built once by `main` after login.
#[derive(Clone)]
pub struct Setup {
    pub client: PolitoClient,
    pub base_url: String,
}

impl Setup {
    /// Environment variable a test cannot run without.
    pub fn require(&self, name: &str) -> Result<String> {
        std::env::var(name).with_context(|| format!("set {name} in .env.test to run this test"))
    }
}
