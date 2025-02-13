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

// Re-export modules
pub mod package_managers {
    pub mod bun;
    pub mod deno;
    pub mod npm;
    pub mod pnpm;
    pub mod pnpm6;
    pub mod yarn;
    pub mod yarn_berry;
}

pub mod detect;
pub mod parse;
pub mod runner;
#[cfg(test)]
mod tests;

mod command_executor;
pub use command_executor::execute_command;
