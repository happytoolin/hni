// src/yarn.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct YarnExecutor {}

impl CommandExecutor for YarnExecutor {
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

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["upgrade".to_string(), args.join(" ").to_string()],
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["remove".to_string(), args.join(" ").to_string()],
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["install".to_string(), "--frozen-lockfile".to_string()],
        })
    }
}

pub struct YarnFactory {}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {})
    }
}
