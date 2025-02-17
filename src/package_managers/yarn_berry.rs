// src/yarn_berry.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct YarnBerryExecutor {}

impl CommandExecutor for YarnBerryExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["run".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["install".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["add".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["exec".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["up".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["remove".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["install".to_string(), "--immutable".to_string()],
        })
    }
}

pub struct YarnBerryFactory {}

impl PackageManagerFactory for YarnBerryFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnBerryExecutor {})
    }
}
