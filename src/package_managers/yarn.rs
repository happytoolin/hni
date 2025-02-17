// src/yarn.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct YarnExecutor {}

impl CommandExecutor for YarnExecutor {
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
        let mut command_args = vec!["upgrade".to_string()];
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
            args: vec!["install".to_string(), "--frozen-lockfile".to_string()],
        })
    }
}

pub struct YarnFactory {}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils::{test_basic_commands, test_edge_cases};

    const YARN_COMMANDS: &[(&str, &str)] = &[
        ("run", "run"),
        ("install", "install"),
        ("add", "add"),
        ("uninstall", "remove"),
        ("clean_install", "install"),
        ("clean_install_args", "--frozen-lockfile"),
    ];

    #[test]
    fn test_yarn_basic_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnFactory {});
        test_basic_commands(factory, "yarn", YARN_COMMANDS);
    }

    #[test]
    fn test_yarn_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnFactory {});
        test_edge_cases(factory, "yarn", YARN_COMMANDS);
    }

    #[test]
    fn test_yarn_specific_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(YarnFactory {});
        let executor = factory.create_commands();

        // Test yarn exec
        let command = executor.execute(vec!["vite"]).unwrap();
        assert_eq!(command.bin, "yarn");
        assert_eq!(command.args, vec!["exec", "vite"]);

        // Test yarn upgrade
        let command = executor.upgrade(vec![]).unwrap();
        assert_eq!(command.bin, "yarn");
        assert_eq!(command.args, vec!["upgrade"]);
    }
}
