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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils::{test_basic_commands, test_edge_cases};

    const YARN_BERRY_COMMANDS: &[(&str, &str)] = &[
        ("run", "run"),
        ("install", "install"),
        ("add", "add"),
        ("uninstall", "remove"),
        ("clean_install", "install"),
        ("clean_install_args", "--immutable"),
    ];

    #[test]
    fn test_yarn_berry_basic_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnBerryFactory {});
        test_basic_commands(factory, "yarn", YARN_BERRY_COMMANDS);
    }

    #[test]
    fn test_yarn_berry_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnBerryFactory {});
        test_edge_cases(factory, "yarn", YARN_BERRY_COMMANDS);
    }

    #[test]
    fn test_yarn_berry_specific_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnBerryFactory {});
        let executor = factory.create_commands();

        // Test yarn exec
        let command = executor.execute(vec!["vite"]).unwrap();
        assert_eq!(command.bin, "yarn");
        assert_eq!(command.args, vec!["exec", "vite"]);

        // Test yarn up
        let command = executor.upgrade(vec![]).unwrap();
        assert_eq!(command.bin, "yarn");
        assert_eq!(command.args, vec!["up"]);
    }
}
