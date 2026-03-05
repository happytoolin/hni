use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub last_run_command: Option<String>,
}

pub fn load_storage() -> Result<Storage> {
    let path = storage_path()?;
    if !path.exists() {
        return Ok(Storage::default());
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read storage file {}", path.display()))?;
    let parsed = serde_json::from_str::<Storage>(&raw)
        .with_context(|| format!("failed to parse storage file {}", path.display()))?;

    Ok(parsed)
}

pub fn save_storage(storage: &Storage) -> Result<()> {
    let path = storage_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create storage dir {}", parent.display()))?;
    }

    let raw = serde_json::to_string(storage)?;
    fs::write(&path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn storage_path() -> Result<PathBuf> {
    let base = dirs::cache_dir().ok_or_else(|| anyhow::anyhow!("cannot find cache directory"))?;
    Ok(base.join("hni").join("storage.json"))
}
