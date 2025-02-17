use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct NirsConfig {
    #[serde(default)]
    pub default_package_manager: Option<String>,
}

pub fn load(cwd: &Path) -> Result<NirsConfig> {
    let config = Config::builder()
        .add_source(
            ["nirs.toml", "nirs.json", "nirs.yaml"]
                .iter()
                .filter_map(|f| {
                    let path = cwd.join(f);
                    match fs::metadata(&path) {
                        Ok(meta) if meta.is_file() && meta.len() > 0 => {
                            Some(File::from(path).required(false))
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>(),
        )
        .build()
        .context("Failed to build config")?;

    config
        .try_deserialize()
        .context("Failed to deserialize config")
}
