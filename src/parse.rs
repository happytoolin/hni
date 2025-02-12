use crate::detect::PackageManager;

pub struct ResolvedCommand {
    pub bin: String,
    pub args: Vec<String>,
}

pub fn parse_ni(agent: PackageManager, args: &[String]) -> ResolvedCommand {
    match agent {
        PackageManager::Npm => {
            let args_str = args.join(" ");
            ResolvedCommand {
                bin: "npm".into(),
                args: vec!["install".into(), args_str],
            }
        }
        // Additional mapping for Yarn, Pnpm, etc.
    }
}
