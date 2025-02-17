// src/npm.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct NpmExecutor {}

impl CommandExecutor for NpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["run".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["install".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: command_args,
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["install".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let command_args = args.iter().map(|s| s.to_string()).collect();

        Some(ResolvedCommand {
            bin: "npx".to_string(),
            args: command_args,
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["update".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: command_args,
        })
    }

    fn uninstall(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["uninstall".to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: command_args,
        })
    }

    fn clean_install(&self, _args: Vec<&str>) -> Option<ResolvedCommand> {
        Some(ResolvedCommand {
            bin: "npm".to_string(),
            args: vec!["ci".to_string()],
        })
    }
}

pub struct NpmFactory {}

impl PackageManagerFactory for NpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(NpmExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils::{test_basic_commands, test_edge_cases};

    const NPM_COMMANDS: &[(&str, &str)] = &[
        ("run", "run"),
        ("install", "install"),
        ("add", "install"),
        ("uninstall", "uninstall"),
        ("clean_install", "ci"),
    ];

    #[test]
    fn test_npm_basic_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
        test_basic_commands(factory, "npm", NPM_COMMANDS);
    }

    #[test]
    fn test_npm_edge_cases() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
        test_edge_cases(factory, "npm", NPM_COMMANDS);
    }

    #[test]
    fn test_npm_specific_commands() {
        let factory: Box<dyn PackageManagerFactory> = Box::new(NpmFactory {});
        let executor = factory.create_commands();

        // Test npx execution
        let command = executor.execute(vec!["vite"]).unwrap();
        assert_eq!(command.bin, "npx");
        assert_eq!(command.args, vec!["vite"]);

        // Test npm update
        let command = executor.upgrade(vec![]).unwrap();
        assert_eq!(command.bin, "npm");
        assert_eq!(command.args, vec!["update"]);
    }
}
