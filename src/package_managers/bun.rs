// src/bun.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct BunExecutor {}

impl CommandExecutor for BunExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["run".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["install".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: command_args,
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["add".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["x".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: command_args,
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["update".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: command_args,
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["remove".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: command_args,
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "bun".to_string(),
            args: vec!["install".to_string(), "--frozen-lockfile".to_string()],
        })
    }
}

pub struct BunFactory {}

impl PackageManagerFactory for BunFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(BunExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils::{test_basic_commands, test_edge_cases};

    const BUN_COMMANDS: &[(&str, &str)] = &[
        ("run", "run"),
        ("install", "install"),
        ("add", "add"),
        ("uninstall", "remove"),
        ("clean_install", "install"),
        ("clean_install_args", "--frozen-lockfile"),
    ];

    #[test]
    fn test_bun_basic_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(BunFactory {});
        test_basic_commands(factory, "bun", BUN_COMMANDS);
    }

    #[test]
    fn test_bun_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(BunFactory {});
        test_edge_cases(factory, "bun", BUN_COMMANDS);
    }

    #[test]
    fn test_bun_specific_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(BunFactory {});
        let executor = factory.create_commands();

        // Test bun x
        let command = executor.execute(vec!["vite"]).unwrap();
        assert_eq!(command.bin, "bun");
        assert_eq!(command.args, vec!["x", "vite"]);

        // Test bun update
        let command = executor.upgrade(vec![]).unwrap();
        assert_eq!(command.bin, "bun");
        assert_eq!(command.args, vec!["update"]);
    }
}
