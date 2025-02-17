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

#[derive(Clone)]
pub struct BunFactory {}

impl PackageManagerFactory for BunFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(BunExecutor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package_managers::test_utils;

    #[test]
    fn test_bun_commands() {
        let factory = BunFactory {};
        let command_map = vec![
            ("run", "run"),
            ("install", "install"),
            ("add", "add"),
            ("add_dev_flag", "--dev"),
            ("execute", "x"),
            ("clean_install_args", "--frozen-lockfile"),
            ("upgrade", "update"),
            ("uninstall", "remove"),
        ];

        test_utils::test_basic_commands(Box::new(factory.clone()), "bun", &command_map);
        test_utils::test_edge_cases(Box::new(factory), "bun", &command_map);
    }
}
