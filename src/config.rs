use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct NirsConfig {
    #[serde(default)]
    pub default_package_manager: Option<String>,
}

pub fn load(cwd: &Path) -> Result<NirsConfig> {
    let config = Config::builder()
        .add_source(
            File::with_name(&cwd.join("nirs").display().to_string())
                .required(false)
                .format(config::FileFormat::Json),
        )
        .add_source(
            File::with_name(&cwd.join("nirs").display().to_string())
                .required(false)
                .format(config::FileFormat::Toml),
        )
        .add_source(
            File::with_name(&cwd.join("nirs").display().to_string())
                .required(false)
                .format(config::FileFormat::Yaml),
        )
        .build()
        .context("Failed to build config")?;

    config
        .try_deserialize()
        .context("Failed to deserialize config")
}
