// src/deno.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub fn deno_execute(args: Vec<&str>) -> Option<ResolvedCommand> {
    Some(ResolvedCommand {
        bin: "deno".to_string(),
        args: vec!["run".to_string(), format!("npm:{}", args[0])],
    })
}

pub struct DenoExecutor {}

impl CommandExecutor for DenoExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["task".to_string(), args.join(" ").to_string()],
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["add".to_string(), args.join(" ").to_string()],
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["run".to_string(), format!("npm:{}", args[0])],
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        // Deno doesn't have a direct "update" command.  Needs workaround.
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["update".to_string(), args.join(" ").to_string()],
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        // Deno doesn't have a direct "remove" command. Needs workaround.
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["remove".to_string(), args.join(" ").to_string()],
        })
    }

    fn clean_install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["install".to_string(), args.join(" ").to_string()],
        })
    }
}

pub struct DenoFactory {}

impl PackageManagerFactory for DenoFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(DenoExecutor {})
    }
}
