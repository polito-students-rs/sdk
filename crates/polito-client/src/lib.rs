#![allow(clippy::too_many_arguments)]

pub mod auth;
mod client;
mod error;
mod middleware;
mod session;

pub use client::{PolitoClient, PolitoClientBuilder};
pub use error::{ApiError, Result};
pub use polito_api::models::{Client as AppInfo, Device, update_preferences_request::Language};
pub use session::Session;
