use crate::{detect::PackageManagerFactoryEnum, CommandExecutor, ResolvedCommand};
use anyhow::{anyhow, Result};

pub struct PackageManager {
    pub name: PackageManagerFactoryEnum,
    pub executor: Box<dyn CommandExecutor>,
}

impl PackageManagerFactoryEnum {
    pub fn create_package_manager(&self) -> PackageManager {
        let executor = self.get_factory().create_commands();
        PackageManager {
            name: *self,
            executor,
        }
    }
}

fn parse_command(
    agent: PackageManagerFactoryEnum,
    args: &[String],
    command: fn(&dyn CommandExecutor, Vec<&str>) -> Option<ResolvedCommand>,
) -> Result<ResolvedCommand> {
    let package_manager = agent.create_package_manager();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    command(package_manager.executor.as_ref(), args).ok_or(anyhow!("Failed to execute command"))
}

pub fn parse_ni(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    parse_command(agent, args, |executor, args| executor.add(args)).unwrap()
}

pub fn parse_nu(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    parse_command(agent, args, |executor, args| executor.upgrade(args)).unwrap()
}

pub fn parse_nun(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    parse_command(agent, args, |executor, args| executor.uninstall(args)).unwrap()
}

pub fn parse_nci(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    parse_command(agent, args, |executor, args| executor.clean_install(args)).unwrap()
}

pub fn parse_na(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    parse_command(agent, args, |executor, args| executor.run(args)).unwrap()
}

pub fn parse_nlx(agent: PackageManagerFactoryEnum, args: &[String]) -> Result<ResolvedCommand> {
    let package_manager = agent.create_package_manager();
    let command_args = agent.get_nlx_command();

    package_manager
        .executor
        .execute(
            command_args
                .into_iter()
                .chain(args.iter().map(|s| s.as_str()))
                .collect(),
        )
        .ok_or(anyhow!("Failed to execute command"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::PackageManagerFactoryEnum;

    #[test]
    fn test_parse_ni() {
        let agents = [
            (PackageManagerFactoryEnum::Npm, vec!["install"]),
            (PackageManagerFactoryEnum::Yarn, vec!["add"]),
            (PackageManagerFactoryEnum::Pnpm, vec!["add"]),
            (PackageManagerFactoryEnum::Bun, vec!["add"]),
        ];

        for (agent, expected) in agents {
            let args = vec!["lodash".to_string()];
            let command = parse_ni(agent, &args);
            assert_eq!(command.args, [expected, vec!["lodash"]].concat());
        }
    }

    #[test]
    fn test_parse_nu() {
        let agents = [
            (PackageManagerFactoryEnum::Npm, vec!["update"]),
            (PackageManagerFactoryEnum::Yarn, vec!["upgrade"]),
            (PackageManagerFactoryEnum::Pnpm, vec!["update"]),
            (PackageManagerFactoryEnum::Bun, vec!["update"]),
        ];

        for (agent, expected) in agents {
            let args = vec!["react".to_string()];
            let command = parse_nu(agent, &args);
            assert_eq!(command.args, [expected, vec!["react"]].concat());
        }
    }

    #[test]
    fn test_parse_nun() {
        let agents = [
            (PackageManagerFactoryEnum::Npm, vec!["uninstall"]),
            (PackageManagerFactoryEnum::Yarn, vec!["remove"]),
            (PackageManagerFactoryEnum::Pnpm, vec!["remove"]),
            (PackageManagerFactoryEnum::Bun, vec!["remove"]),
        ];

        for (agent, expected) in agents {
            let args = vec!["vue".to_string()];
            let command = parse_nun(agent, &args);
            assert_eq!(command.args, [expected, vec!["vue"]].concat());
        }
    }

    #[test]
    fn test_parse_nci() {
        let agents = [
            (PackageManagerFactoryEnum::Npm, vec!["ci"]),
            (
                PackageManagerFactoryEnum::Yarn,
                vec!["install", "--frozen-lockfile"],
            ),
            (
                PackageManagerFactoryEnum::Pnpm,
                vec!["install", "--frozen-lockfile"],
            ),
            (
                PackageManagerFactoryEnum::Bun,
                vec!["install", "--frozen-lockfile"],
            ),
        ];

        for (agent, expected) in agents {
            let args = vec![];
            let command = parse_nci(agent, &args);
            assert_eq!(command.args, expected);
        }
    }

    #[test]
    fn test_parse_na() {
        let agents = [
            (PackageManagerFactoryEnum::Npm, vec!["run"]),
            (PackageManagerFactoryEnum::Yarn, vec!["run"]),
            (PackageManagerFactoryEnum::Pnpm, vec!["run"]),
            (PackageManagerFactoryEnum::Bun, vec!["run"]),
        ];

        for (agent, expected) in agents {
            let args = vec!["build".to_string()];
            let command = parse_na(agent, &args);
            assert_eq!(command.args, [expected, vec!["build"]].concat());
        }
    }

    #[test]
    fn test_parse_nlx() {
        let agents = [
            (PackageManagerFactoryEnum::Npm, vec!["npx"]),
            (PackageManagerFactoryEnum::Yarn, vec!["exec", "yarn", "dlx"]),
            (PackageManagerFactoryEnum::Pnpm, vec!["dlx", "pnpm"]),
            (PackageManagerFactoryEnum::Bun, vec!["x", "bun"]),
        ];

        for (agent, expected) in agents {
            let args = vec!["vitest".to_string()];
            let command = parse_nlx(agent, &args);
            let expected_args: Vec<&str> = expected
                .iter()
                .chain(vec!["vitest"].iter())
                .map(|s| *s)
                .collect();
            assert_eq!(command.unwrap().args, expected_args);
        }
    }
}
