use std::path::PathBuf;

use crate::core::config::HniConfig;

#[derive(Debug, Clone)]
pub struct ResolveContext {
    pub cwd: PathBuf,
    pub config: HniConfig,
}

impl ResolveContext {
    pub fn new(cwd: PathBuf, config: HniConfig) -> Self {
        Self { cwd, config }
    }
}
