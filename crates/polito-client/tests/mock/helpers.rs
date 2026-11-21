use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn workspace_root() -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    root.canonicalize()
        .with_context(|| format!("cannot resolve workspace root {}", root.display()))
}

pub fn load_dotenv() -> Result<()> {
    let root = workspace_root()?;
    for name in [".env.test", ".env"] {
        let path = root.join(name);
        if path.exists() {
            dotenvy::from_path(&path).with_context(|| format!("cannot load {}", path.display()))?;
        }
    }
    Ok(())
}
