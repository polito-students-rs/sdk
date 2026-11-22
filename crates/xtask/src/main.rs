pub mod cli;
pub mod overlay;
pub mod sync_version;

use cli::Cli;
use anyhow::Result;

fn main() -> Result<()> {
    Cli::process()
}