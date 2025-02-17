// src/pnpm.rs

use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

// Modern pnpm (v7+)
pub struct PnpmModernExecutor {}

impl CommandExecutor for PnpmModernExecutor {
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
        command_args.extend(args.iter().map(|s| {
            match *s {
                "-D" => "--save-dev",
                s => s,
            }
            .to_string()
        }));

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

// Legacy pnpm (v6)
pub struct PnpmLegacyExecutor {}

impl CommandExecutor for PnpmLegacyExecutor {
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
        command_args.extend(args.iter().map(|s| {
            match *s {
                "-D" => "--save-dev",
                s => s,
            }
            .to_string()
        }));

        Some(ResolvedCommand {
            bin: "pnpm".to_string(),
            args: command_args,
        })
    }

    fn execute(&self, args: Vec<&str>) -> Option<ResolvedCommand> {
        let mut command_args = vec!["exec".to_string()];
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

// Replace the separate factories with a unified version-aware factory
#[derive(Clone)]
pub struct PnpmFactory {
    pub is_legacy: bool,
}

impl PnpmFactory {
    pub fn new(is_legacy: bool) -> Self {
        Self { is_legacy }
    }
}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        if self.is_legacy {
            Box::new(PnpmLegacyExecutor {})
        } else {
            Box::new(PnpmModernExecutor {})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    // Test both versions
    fn test_pnpm_version(version: &str, execute_cmd: &str) {
        let factory = PnpmFactory {
            is_legacy: version == "legacy",
        };

        let command_map = vec![
            ("run", "run"),
            ("install", "install"),
            ("add", "add"),
            ("add_dev_flag", "--save-dev"),
            ("execute", execute_cmd),
            ("clean_install_args", "--frozen-lockfile"),
            ("upgrade", "update"),
            ("uninstall", "remove"),
        ];

        test_utils::test_basic_commands(Box::new(factory.clone()), "pnpm", &command_map);
        test_utils::test_edge_cases(Box::new(factory), "pnpm", &command_map);
    }

    #[test]
    fn test_pnpm_modern() {
        test_pnpm_version("modern", "dlx");
    }

    #[test]
    fn test_pnpm_legacy() {
        test_pnpm_version("legacy", "exec");
    }
}
