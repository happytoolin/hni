use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::{Arg, ArgAction, Command};

#[derive(Debug, Clone)]
pub struct ParsedFlags {
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub debug: bool,
    pub show_help: bool,
    pub show_version: bool,
}

pub fn parse_global_flags(base_cwd: &Path, args: Vec<String>) -> Result<ParsedFlags> {
    let argv = std::iter::once("hni".to_string())
        .chain(args)
        .collect::<Vec<_>>();

    let matches = global_flags_command()
        .try_get_matches_from(argv)
        .map_err(|e| anyhow!(e.to_string()))?;

    let mut cwd = base_cwd.to_path_buf();
    for path in matches.get_many::<PathBuf>("cwd").into_iter().flatten() {
        cwd = cwd.join(path);
    }

    let mut rest = matches
        .get_many::<String>("args")
        .map(|vals| vals.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut debug = false;
    rest.retain(|arg| {
        if matches!(arg.as_str(), "?" | "-?" | "--debug-resolved") {
            debug = true;
            false
        } else {
            true
        }
    });

    let show_help = rest.len() == 1 && matches!(rest[0].as_str(), "-h" | "--help");
    let show_version = rest.len() == 1 && matches!(rest[0].as_str(), "-v" | "--version");

    Ok(ParsedFlags {
        cwd,
        args: rest,
        debug,
        show_help,
        show_version,
    })
}

fn global_flags_command() -> Command {
    Command::new("hni")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("cwd")
                .short('C')
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf))
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("args")
                .num_args(0..)
                .allow_hyphen_values(true)
                .action(ArgAction::Append),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cwd_and_debug() {
        let base = PathBuf::from("/tmp");
        let parsed = parse_global_flags(
            &base,
            vec!["-C".into(), "proj".into(), "dev".into(), "?".into()],
        )
        .unwrap();

        assert_eq!(parsed.cwd, PathBuf::from("/tmp").join("proj"));
        assert_eq!(parsed.args, vec!["dev"]);
        assert!(parsed.debug);
    }

    #[test]
    fn preserves_unknown_flags_as_args() {
        let base = PathBuf::from("/tmp");
        let parsed =
            parse_global_flags(&base, vec!["-p".into(), "1+1".into(), "?".into()]).unwrap();

        assert_eq!(parsed.args, vec!["-p", "1+1"]);
        assert!(parsed.debug);
    }
}
