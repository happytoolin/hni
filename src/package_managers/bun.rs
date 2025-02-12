// src/bun.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct BunExecutor {}

impl CommandExecutor for BunExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: vec!["run".to_string(), args.join(" ").to_string()],
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: vec!["add".to_string(), args.join(" ").to_string()],
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: vec!["x".to_string(), args.join(" ").to_string()],
        })
    }
}

pub struct BunFactory {}

impl PackageManagerFactory for BunFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(BunExecutor {})
    }
}
