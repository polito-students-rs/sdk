pub mod cli;
pub mod sync_version;

use cli::Cli;
use anyhow::Result;

fn main() -> Result<()> {
    Cli::process()
}