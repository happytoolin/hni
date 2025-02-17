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
        command_args.extend(args.iter().map(|s| {
            match *s {
                "-D" => "--save-dev",
                s => s,
            }
            .to_string()
        }));

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
            args: vec!["ci".to_string(), "--frozen-lockfile".to_string()],
        })
    }
}

#[derive(Clone)]
pub struct NpmFactory {}

impl PackageManagerFactory for NpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(NpmExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    #[test]
    fn test_npm_commands() {
        let factory = NpmFactory {};
        let command_map = vec![
            ("run", "run"),
            ("install", "install"),
            ("add", "install"),
            ("add_dev_flag", "--save-dev"),
            ("execute", "npx"),
            ("clean_install", "ci"),
            ("upgrade", "update"),
            ("uninstall", "uninstall"),
        ];

        test_utils::test_basic_commands(Box::new(factory.clone()), "npm", &command_map);
        test_utils::test_edge_cases(Box::new(factory), "npm", &command_map);
    }
}
