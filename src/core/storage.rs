use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub last_run_command: Option<String>,
}

pub fn load_storage() -> Result<Storage> {
    confy::load("hni", Some("storage")).context("failed to load hni storage")
}

pub fn save_storage(storage: &Storage) -> Result<()> {
    confy::store("hni", Some("storage"), storage).context("failed to save hni storage")
}
