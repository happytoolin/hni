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
        let mut command_args = vec!["task".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["install".to_string()];
        if args.contains(&"-g") {
            command_args.push("--allow-run".to_string());
            command_args.push("-n".to_string());
        }
        command_args.extend(
            args.iter()
                .filter(|&&arg| arg != "-g")
                .map(|s| s.to_string()),
        );

        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: command_args,
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        // Deno doesn't have a direct add command, it uses imports in code
        // For global tools, we use install
        if args.contains(&"-D") {
            // Dev dependencies are typically managed in dev_deps.ts
            None
        } else {
            let mut command_args = vec!["install".to_string()];
            command_args.extend(args.iter().map(|s| s.to_string()));
            Some(ResolvedCommand {
                bin: "deno".to_string(),
                args: command_args,
            })
        }
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["run".to_string()];
        if args.len() > 0 {
            command_args.push(format!("npm:{}", args[0]));
            if args.len() > 1 {
                command_args.extend(args[1..].iter().map(|s| s.to_string()));
            }
        }

        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: command_args,
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["upgrade".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: command_args,
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        // Deno doesn't have a direct uninstall command
        // For global tools, we can use manual removal
        None
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["cache".to_string(), "deps.ts".to_string()],
        })
    }
}

pub struct DenoFactory {}

impl PackageManagerFactory for DenoFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(DenoExecutor {})
    }
}
