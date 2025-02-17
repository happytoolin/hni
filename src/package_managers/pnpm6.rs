// src/pnpm6.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct Pnpm6Executor {}

impl CommandExecutor for Pnpm6Executor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["run".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["install".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["add".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["dlx".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["update".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["remove".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils::{test_basic_commands, test_edge_cases};

    const PNPM6_COMMANDS: &[(&str, &str)] = &[
        ("run", "run"),
        ("install", "install"),
        ("add", "add"),
        ("uninstall", "remove"),
        ("clean_install", "install"),
        ("clean_install_args", "--frozen-lockfile"),
    ];

    #[test]
    fn test_pnpm6_basic_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(Pnpm6Factory {});
        test_basic_commands(factory, "pnpm", PNPM6_COMMANDS);
    }

    #[test]
    fn test_pnpm6_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(Pnpm6Factory {});
        test_edge_cases(factory, "pnpm", PNPM6_COMMANDS);
    }

    #[test]
    fn test_pnpm6_specific_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(Pnpm6Factory {});
        let executor = factory.create_commands();

        // Test pnpm exec
        let command = executor.execute(vec!["vite"]).unwrap();
        assert_eq!(command.bin, "pnpm");
        assert_eq!(command.args, vec!["exec", "vite"]);

        // Test pnpm update
        let command = executor.upgrade(vec![]).unwrap();
        assert_eq!(command.bin, "pnpm");
        assert_eq!(command.args, vec!["update"]);
    }
}
