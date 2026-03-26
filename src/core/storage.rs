use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Storage {
    pub last_run_command: Option<String>,
}

fn storage_file() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("storage error: failed to resolve config directory"))?;
    Ok(config_dir
        .join("hni")
        .join("storage")
        .join("last-run-command"))
}

pub fn load_storage() -> Result<Storage> {
    let path = storage_file()?;
    if !path.exists() {
        return Ok(Storage::default());
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| anyhow!("storage error: failed to load hni storage: {error}"))?;
    let trimmed = raw.trim();

    Ok(Storage {
        last_run_command: (!trimmed.is_empty()).then(|| trimmed.to_string()),
    })
}

pub fn save_storage(storage: &Storage) -> Result<()> {
    let path = storage_file()?;
    let contents = storage
        .last_run_command
        .as_deref()
        .map(|value| format!("{value}\n"))
        .unwrap_or_default();

    match fs::read_to_string(&path) {
        Ok(existing) if existing == contents => return Ok(()),
        Ok(_) | Err(_) => {}
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| anyhow!("storage error: failed to save hni storage: {error}"))?;
    }

    fs::write(path, contents)
        .map_err(|error| anyhow!("storage error: failed to save hni storage: {error}"))
}
