// src/pnpm6.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct Pnpm6Executor {}

impl CommandExecutor for Pnpm6Executor {
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

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["update".to_string(), args.join(" ").to_string()],
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["remove".to_string(), args.join(" ").to_string()],
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: vec!["install".to_string(), "--frozen-lockfile".to_string()],
        })
    }
}

pub struct Pnpm6Factory {}

impl PackageManagerFactory for Pnpm6Factory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(Pnpm6Executor {})
    }
}
