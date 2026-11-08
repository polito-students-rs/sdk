use askama::Template;
use std::io::Write;
use std::path::Path;
use std::fs::{self, OpenOptions};
use fs_utils::copy;
use anyhow::{Result, anyhow};
use cargo_toml::Manifest;


#[derive(Template)]
#[template(path = "rust.yml")]
struct RustGeneratorTemplate<'a> {
    package_version: &'a str
}

fn retrieve_polito_client_version() -> Result<String> {
    let cargo_toml_path = Path::new("crates")
        .join("polito-client")
        .join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow!("{} does not exist", cargo_toml_path.to_string_lossy()));
    }
    let manifest = Manifest::from_path(cargo_toml_path)?;
    let package: cargo_toml::Package = manifest.package.ok_or(anyhow!("cargo toml is incomplete"))?;
    Ok(package.version.get()?.to_string())
}

pub fn render_rust_gen_config() -> Result<()> {
    let out_dir = Path::new("target").join("codegen");
    let shared_dir = Path::new("spec")
        .join("codegen")
        .join("shared");
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    } else {
        fs::remove_dir_all(&out_dir)?;
    }
    copy::copy_directory(shared_dir, &out_dir)?;
    let polito_client_version = retrieve_polito_client_version()?;
    let content = RustGeneratorTemplate { package_version: polito_client_version.as_str() }.render()?;
    let mut rust_gen_config = OpenOptions::new()
        .write(true)
        .create(true)
        .open(out_dir.join("rust.yml"))?;
    rust_gen_config.write(content.as_bytes())?;
    Ok(())
}