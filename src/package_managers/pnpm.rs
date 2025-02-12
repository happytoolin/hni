// src/pnpm.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct PnpmExecutor {}

impl CommandExecutor for PnpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["run".to_string(), args.join(" ").to_string()],
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["add".to_string(), args.join(" ").to_string()],
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["dlx".to_string(), args.join(" ").to_string()],
        })
    }
}

pub struct PnpmFactory {}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(PnpmExecutor {})
    }
}
