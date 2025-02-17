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
            command_args.push("-g".to_string());
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
        let mut command_args = vec!["add".to_string()];
        command_args.extend(args.iter().map(|s| match *s {
            "-D" | "--save-dev" => "--save-dev".to_string(),
            s => s.to_string(),
        }));
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["run".to_string()];
        if !args.is_empty() {
            command_args.extend(args.iter().map(|s| s.to_string()));
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
        // As of Deno 2.0, deno remove is the equivalent
        let mut command_args = vec!["remove".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: command_args,
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "deno".to_string(),
            args: vec!["cache".to_string(), "--frozen-lockfile".to_string()],
        })
    }
}

#[derive(Clone)]
pub struct DenoFactory {}

impl PackageManagerFactory for DenoFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(DenoExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    #[test]
    fn test_deno_commands() {
        let factory = DenoFactory {};
        let command_map = vec![
            ("run", "task"),            // npm run -> deno task
            ("install", "install"),     // npm install -> deno install
            ("clean_install", "cache"), // npm ci -> deno cache
            ("upgrade", "upgrade"),     // npm update -> deno upgrade
            ("add", "add"),             // npm add -> deno add (Deno 2.0)
            ("uninstall", "remove"),    // npm uninstall -> deno remove
        ];

        // Basic commands
        test_utils::test_basic_commands(Box::new(factory.clone()), "deno", &command_map);

        // Deno-specific edge cases
        let executor = factory.create_commands();

        // Test global install
        let command = executor.install(vec!["-g", "my_script"]).unwrap();
        assert_eq!(command.args, vec!["install", "-g", "my_script"]);

        // Test execute (run)
        let command = executor.execute(vec!["script.ts"]).unwrap();
        assert_eq!(command.bin, "deno");
        assert_eq!(command.args, vec!["run", "script.ts"]);
    }
}
