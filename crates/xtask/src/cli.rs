use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use crate::overlay::apply_overlays;
use crate::sync_version::render_rust_gen_config;

/// Wrapper struct that encapsule all command
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {

    #[command(subcommand)]
    command: Option<Commands>,


}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Render target/codegen/*.yml from spec/codegen with the polito-client version
    SyncVersion,
    /// Apply spec/overlays/*.yaml to spec/openapi.yaml into target/codegen/openapi.yaml
    ApplyOverlays,
}

impl Cli {
    pub fn process() -> Result<()> {
        let cli = Self::parse();
        match &cli.command {
            Some(Commands::SyncVersion) => {
                render_rust_gen_config()
            },
            Some(Commands::ApplyOverlays) => apply_overlays(),
            None => Ok(())
        }
    }
}