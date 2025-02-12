use crate::detect::PackageManagerFactoryEnum;
use crate::ResolvedCommand;

fn get_command(
    agent: PackageManagerFactoryEnum,
    command: &str,
    args: &[String],
) -> ResolvedCommand {
    println!("Agent: {:?}, Command: {}, Args: {:?}", agent, command, args);
    let args_str = args.join(" ");
    match agent {
        PackageManagerFactoryEnum::Npm => {
            println!("Using npm");
            ResolvedCommand {
                bin: "npm".into(),
                args: vec!["run".into(), args_str],
            }
        }
        PackageManagerFactoryEnum::Yarn => ResolvedCommand {
            bin: "yarn".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Pnpm => ResolvedCommand {
            bin: "pnpm".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Bun => ResolvedCommand {
            bin: "bun".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Deno => ResolvedCommand {
            bin: "deno".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::YarnBerry => ResolvedCommand {
            bin: "yarn".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Pnpm6 => ResolvedCommand {
            bin: "pnpm".into(),
            args: vec!["run".into(), args_str],
        },
    }
}

pub fn parse_ni(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    println!("Agent: {:?}, Args: {:?}", agent, args);
    let args_str = args.join(" ");
    let command = match agent {
        PackageManagerFactoryEnum::Npm => {
            println!("Using npm");
            ResolvedCommand {
                bin: "npm".into(),
                args: vec!["run".into(), args_str],
            }
        }
        PackageManagerFactoryEnum::Yarn => ResolvedCommand {
            bin: "yarn".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Pnpm => ResolvedCommand {
            bin: "pnpm".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Bun => ResolvedCommand {
            bin: "bun".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Deno => ResolvedCommand {
            bin: "deno".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::YarnBerry => ResolvedCommand {
            bin: "yarn".into(),
            args: vec!["run".into(), args_str],
        },
        PackageManagerFactoryEnum::Pnpm6 => ResolvedCommand {
            bin: "pnpm".into(),
            args: vec!["run".into(), args_str],
        },
    };
    println!("Command: {:?}", command);
    command
}

pub fn parse_nu(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    if args.contains(&"-i".to_string()) {
        let mut new_args = args.to_vec();
        new_args.retain(|x| x != "-i");
        get_command(agent, "upgrade-interactive", &new_args)
    } else {
        get_command(agent, "upgrade", args)
    }
}

pub fn parse_nun(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    if args.contains(&"-g".to_string()) {
        let mut new_args = args.to_vec();
        new_args.retain(|x| x != "-g");
        get_command(agent, "global_uninstall", &new_args)
    } else {
        get_command(agent, "uninstall", args)
    }
}

pub fn parse_nci(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    let mut new_args = args.to_vec();
    new_args.push("--frozen-if-present".to_string());
    parse_ni(agent, &new_args)
}

pub fn parse_na(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    get_command(agent, "agent", args)
}

pub fn parse_nlx(agent: PackageManagerFactoryEnum, args: &[String]) -> ResolvedCommand {
    get_command(agent, "execute", args)
}
