use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
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
    SyncVersion
}

impl Cli {
    pub fn process() -> Result<()> {
        let cli = Self::parse();
        match &cli.command {
            Some(Commands::SyncVersion) => {
                render_rust_gen_config()
            },
            None => Ok(())
        }
    }
}