// src/pnpm.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct PnpmExecutor {}

impl CommandExecutor for PnpmExecutor {
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

pub struct PnpmFactory {}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(PnpmExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils::{test_basic_commands, test_edge_cases};

    const PNPM_COMMANDS: &[(&str, &str)] = &[
        ("run", "run"),
        ("install", "install"),
        ("add", "add"),
        ("uninstall", "remove"),
        ("clean_install", "install"),
        ("clean_install_args", "--frozen-lockfile"),
    ];

    #[test]
    fn test_pnpm_basic_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(PnpmFactory {});
        test_basic_commands(factory, "pnpm", PNPM_COMMANDS);
    }

    #[test]
    fn test_pnpm_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(PnpmFactory {});
        test_edge_cases(factory, "pnpm", PNPM_COMMANDS);
    }

    #[test]
    fn test_pnpm_specific_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(PnpmFactory {});
        let executor = factory.create_commands();

        // Test pnpm dlx
        let command = executor.execute(vec!["vite"]).unwrap();
        assert_eq!(command.bin, "pnpm");
        assert_eq!(command.args, vec!["dlx", "vite"]);

        // Test pnpm update
        let command = executor.upgrade(vec![]).unwrap();
        assert_eq!(command.bin, "pnpm");
        assert_eq!(command.args, vec!["update"]);
    }
}
