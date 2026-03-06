use serde::{Deserialize, Serialize};

use super::error::{HniError, HniResult};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub last_run_command: Option<String>,
}

pub fn load_storage() -> HniResult<Storage> {
    confy::load("hni", Some("storage"))
        .map_err(|error| HniError::storage(format!("failed to load hni storage: {error}")))
}

pub fn save_storage(storage: &Storage) -> HniResult<()> {
    confy::store("hni", Some("storage"), storage)
        .map_err(|error| HniError::storage(format!("failed to save hni storage: {error}")))
}
