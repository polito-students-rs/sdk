use std::path::Path;

use anyhow::{Context, Result};

pub fn load_dotenv() -> Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testing/e2e/.env.test");
    if path.exists() {
        dotenvy::from_path(&path).with_context(|| format!("cannot load {}", path.display()))?;
    }
    Ok(())
}
