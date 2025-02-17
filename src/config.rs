use anyhow::{Context, Result};
use config::{Config, Environment, File};
use dirs_next::config_dir;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct NirsConfig {
    #[serde(default)]
    pub default_package_manager: Option<String>,
}

pub fn load(cwd: &Path) -> Result<NirsConfig> {
    let config_dir =
        config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

    let config = Config::builder()
        .add_source(
            ["nirs.toml", "nirs.json", "nirs.yaml"]
                .iter()
                .filter_map(|f| {
                    let path = cwd.join(f);
                    if path.exists() {
                        match path.metadata() {
                            Ok(meta) if meta.len() > 0 => {
                                Some(Ok(File::from(path).required(false)))
                            }
                            Ok(_) => {
                                log::trace!("Empty config file: {}", f);
                                None
                            }
                            Err(e) => Some(Err(e.into())),
                        }
                    } else {
                        log::trace!("Config file {} not found", f);
                        None
                    }
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
        )
        .add_source(
            ["nirs.toml", "nirs.json", "nirs.yaml"]
                .iter()
                .map(|f| File::from(config_dir.join(f)).required(false))
                .collect::<Vec<_>>(),
        )
        .add_source(Environment::with_prefix("NIRS"))
        .build()
        .context("Failed to build config")?;

    config
        .try_deserialize()
        .context("Failed to deserialize config")
}
