use crate::detect::PackageManagerFactoryEnum;
use crate::ResolvedCommand;

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
