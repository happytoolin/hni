use crate::detect::PackageManager;
use crate::ResolvedCommand;

pub fn parse_ni(agent: PackageManager, args: &[String]) -> ResolvedCommand {
    let args_str = args.join(" ");
    match agent {
        PackageManager::Npm => ResolvedCommand {
            bin: "npm".into(),
            args: vec!["install".into(), args_str],
        },
        PackageManager::Yarn => ResolvedCommand {
            bin: "yarn".into(),
            args: vec!["install".into(), args_str],
        },
        PackageManager::Pnpm => ResolvedCommand {
            bin: "pnpm".into(),
            args: vec!["install".into(), args_str],
        },
        PackageManager::Bun => ResolvedCommand {
            bin: "bun".into(),
            args: vec!["install".into(), args_str],
        },
        PackageManager::Deno => ResolvedCommand {
            bin: "deno".into(),
            args: vec!["install".into(), args_str],
        },
        PackageManager::YarnBerry => ResolvedCommand {
            bin: "yarn".into(),
            args: vec!["install".into(), args_str],
        },
        PackageManager::Pnpm6 => ResolvedCommand {
            bin: "pnpm".into(),
            args: vec!["install".into(), args_str],
        },
    }
}
