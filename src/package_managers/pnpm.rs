use crate::{CommandExecutor, PackageManagerFactory, ResolvedCommand};

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

// Simplify the factory to only support modern pnpm
#[derive(Clone)]
pub struct PnpmFactory;

impl PnpmFactory {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(PnpmModernExecutor {})
    }
}

// Update tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    fn test_pnpm_version() {
        let factory = PnpmFactory::new();

        let command_map = vec![
            ("run", "run"),
            ("install", "install"),
            ("add", "add"),
            ("add_dev_flag", "--save-dev"),
            ("execute", "dlx"),
            ("clean_install_args", "--frozen-lockfile"),
            ("upgrade", "update"),
            ("uninstall", "remove"),
        ];

        test_utils::test_basic_commands(Box::new(factory.clone()), "pnpm", &command_map);
        test_utils::test_edge_cases(Box::new(factory), "pnpm", &command_map);
    }

    #[test]
    fn test_pnpm_modern() {
        test_pnpm_version();
    }
}
