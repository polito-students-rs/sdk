use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn workspace_root() -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    root.canonicalize()
        .with_context(|| format!("cannot resolve workspace root {}", root.display()))
}

/// Loads `testing/integration/.env.test`, if present. Variables already set
/// in the shell win.
pub fn load_dotenv() -> Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testing/integration/.env.test");
    if path.exists() {
        dotenvy::from_path(&path).with_context(|| format!("cannot load {}", path.display()))?;
    }
    Ok(())
}
