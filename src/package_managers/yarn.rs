use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

pub struct YarnExecutor {
    is_berry: bool,
}

impl CommandExecutor for YarnExecutor {
    fn run(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = if self.is_berry {
            vec!["run".to_string()]
        } else {
            vec![]
        };
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn install(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let command_args = if args.contains(&"-g") && !self.is_berry {
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
        if !self.is_berry && args.contains(&"-D") {
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
        let subcommand = if self.is_berry { "exec" } else { "dlx" };
        let mut command_args = vec![subcommand.to_string()];
        command_args.extend(args.iter().map(|s| s.to_string()));

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: command_args,
        })
    }

    fn upgrade(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let subcommand = if self.is_berry { "up" } else { "upgrade" };
        let mut command_args = vec![subcommand.to_string()];
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
        let flag = if self.is_berry {
            "--immutable"
        } else {
            "--frozen-lockfile"
        };

        Some(ResolvedCommand {
            bin: "yarn".to_string(),
            args: vec!["install".to_string(), flag.to_string()],
        })
    }
}

#[derive(Clone)]
pub struct YarnFactory {
    pub is_berry: bool,
}

impl YarnFactory {
    pub fn new(is_berry: bool) -> Self {
        Self { is_berry }
    }
}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {
            is_berry: self.is_berry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    // Unified test function for both versions
    fn test_yarn_version(is_berry: bool) {
        let factory = YarnFactory { is_berry };

        let command_map = vec![
            ("run", if is_berry { "run" } else { "" }),
            ("install", "install"),
            ("add", "add"),
            ("add_dev_flag", if is_berry { "-D" } else { "--dev" }),
            ("execute", if is_berry { "exec" } else { "dlx" }),
            (
                "clean_install_args",
                if is_berry {
                    "--immutable"
                } else {
                    "--frozen-lockfile"
                },
            ),
            ("upgrade", if is_berry { "up" } else { "upgrade" }),
            ("uninstall", "remove"),
            (
                "global_install",
                if is_berry { "install" } else { "global add" },
            ),
        ];

        // Test basic commands
        test_utils::test_basic_commands(Box::new(factory.clone()), "yarn", &command_map);

        // Test version-specific edge cases
        if is_berry {
            test_berry_specific_features(factory.clone());
        } else {
            test_classic_specific_features(factory.clone());
        }
    }

    // Version-specific test cases
    fn test_berry_specific_features(factory: YarnFactory) {
        // Test Berry-specific features like workspaces
        let cmd = factory.create_commands();
        assert_eq!(
            cmd.execute(vec!["eslint"]).unwrap().args,
            vec!["exec", "eslint"]
        );
    }

    fn test_classic_specific_features(factory: YarnFactory) {
        // Test Classic-specific features
        let cmd = factory.create_commands();
        assert_eq!(
            cmd.install(vec!["-g", "typescript"]).unwrap().args,
            vec!["global", "add", "typescript"]
        );
    }

    #[test]
    fn test_yarn_versions() {
        test_yarn_version(false); // Classic
        test_yarn_version(true); // Berry
    }
}
