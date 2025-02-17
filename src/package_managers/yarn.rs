// src/yarn.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct YarnExecutor {}

impl CommandExecutor for YarnExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        // Yarn allows direct script execution without 'run'
        let command_args = args.iter().map(|s| s.to_string()).collect();

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = if args.contains(&"-g") {
            let mut cmd = vec!["global".to_string(), "add".to_string()];
            cmd.extend(
                args.iter()
                    .filter(|&&arg| arg != "-g")
                    .map(|s| s.to_string()),
            );
            cmd
        } else {
            let mut cmd = vec!["install".to_string()];
            cmd.extend(args.iter().map(|s| s.to_string()));
            cmd
        };

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn add(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["add".to_string()];
        if args.contains(&"-D") {
            command_args.extend(args.iter().map(|s| {
                match *s {
                    "-D" => "--dev",
                    s => s,
                }
                .to_string()
            }));
        } else {
            command_args.extend(args.iter().map(|s| s.to_string()));
        }

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["dlx".to_string()];
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

#[derive(Clone)]
pub struct YarnFactory {}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    #[test]
    fn test_yarn_commands() {
        let factory = YarnFactory {};
        let command_map = vec![
            ("run", ""), // yarn doesn't use 'run' keyword
            ("install", "install"),
            ("add", "add"),
            ("add_dev_flag", "--dev"),
            ("execute", "dlx"),
            ("clean_install_args", "--frozen-lockfile"),
            ("upgrade", "upgrade"),
            ("uninstall", "remove"),
            ("global_install", "global add"),
        ];

        test_utils::test_basic_commands(Box::new(factory.clone()), "yarn", &command_map);
        test_utils::test_edge_cases(Box::new(factory), "yarn", &command_map);
    }
}
