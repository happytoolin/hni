// src/npm.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct NpmExecutor {}

impl CommandExecutor for NpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["run".to_string(), args.join(" ").to_string()],
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npx".to_string(),
            args: vec![args.join(" ").to_string()],
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["update".to_string(), args.join(" ").to_string()],
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["uninstall".to_string(), args.join(" ").to_string()],
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["ci".to_string()],
        })
    }
}

pub struct NpmFactory {}

impl PackageManagerFactory for NpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(NpmExecutor {})
    }
}
