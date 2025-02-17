use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCommand {
    pub bin: String,
    pub args: Vec<String>,
}

pub trait CommandExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
    fn clean_install(&self, args: Vec<&str>) -> Option<ResolvedCommand>;
}

pub trait PackageManagerFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor>;
}

pub mod package_managers {
    pub mod bun;
    pub mod deno;
    pub mod npm;
    pub mod pnpm;
    pub mod yarn;

    #[cfg(test)]
    pub(crate) mod test_utils;
}

mod command;
pub use command::NirsCommand;

pub mod detect;
pub mod logger;
pub mod parse;
pub mod runner;

mod command_executor;
pub use command_executor::execute_command;

// Re-export parse functions
pub use crate::parse::{
    parse_na, parse_nci, parse_ni, parse_nlx, parse_nu, parse_nun, PackageManager,
};

// Re-export detect functions
pub use crate::detect::PackageManagerFactoryEnum;

pub mod config;

pub mod update_checker;
