// src/yarn_berry.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct YarnBerryExecutor {}

impl CommandExecutor for YarnBerryExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["run".to_string(), args.join(" ").to_string()],
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["add".to_string(), args.join(" ").to_string()],
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["exec".to_string(), args.join(" ").to_string()],
        })
    }
}

pub struct YarnBerryFactory {}

impl PackageManagerFactory for YarnBerryFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnBerryExecutor {})
    }
}
