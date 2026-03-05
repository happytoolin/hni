use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct ParsedFlags {
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub debug: bool,
    pub explain: bool,
    pub show_help: bool,
    pub show_version: bool,
}

pub fn parse_global_flags(base_cwd: &Path, args: Vec<String>) -> Result<ParsedFlags> {
    let mut cwd = base_cwd.to_path_buf();
    let mut rest = Vec::new();
    let mut debug = false;
    let mut explain = false;
    let mut idx = 0;

    while idx < args.len() {
        let arg = &args[idx];
        match arg.as_str() {
            "-C" | "--cwd" => {
                let Some(path) = args.get(idx + 1) else {
                    return Err(anyhow!("missing value for {arg}"));
                };
                cwd = cwd.join(path);
                idx += 2;
            }
            "?" | "-?" | "--debug-resolved" | "--dry-run" | "--print-command" => {
                debug = true;
                idx += 1;
            }
            "--explain" => {
                explain = true;
                idx += 1;
            }
            _ => {
                if let Some(path) = arg.strip_prefix("-C").filter(|path| !path.is_empty()) {
                    cwd = cwd.join(path);
                } else {
                    rest.push(arg.clone());
                }
                idx += 1;
            }
        }
    }

    let show_help = rest.len() == 1 && matches!(rest[0].as_str(), "-h" | "--help");
    let show_version = rest.len() == 1 && matches!(rest[0].as_str(), "-v" | "--version");

    Ok(ParsedFlags {
        cwd,
        args: rest,
        debug,
        explain,
        show_help,
        show_version,
    })
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
        assert!(!parsed.explain);
    }

    #[test]
    fn preserves_unknown_flags_as_args() {
        let base = PathBuf::from("/tmp");
        let parsed =
            parse_global_flags(&base, vec!["-p".into(), "1+1".into(), "?".into()]).unwrap();

        assert_eq!(parsed.args, vec!["-p", "1+1"]);
        assert!(parsed.debug);
        assert!(!parsed.explain);
    }

    #[test]
    fn parses_explain_aliases() {
        let base = PathBuf::from("/tmp");
        let parsed = parse_global_flags(
            &base,
            vec![
                "--dry-run".into(),
                "--explain".into(),
                "vite".into(),
                "--print-command".into(),
            ],
        )
        .unwrap();

        assert_eq!(parsed.args, vec!["vite"]);
        assert!(parsed.debug);
        assert!(parsed.explain);
    }

    #[test]
    fn parses_cwd_after_subcommand() {
        let base = PathBuf::from("/tmp");
        let parsed = parse_global_flags(
            &base,
            vec!["ni".into(), "-C".into(), "proj".into(), "vite".into()],
        )
        .unwrap();

        assert_eq!(parsed.cwd, PathBuf::from("/tmp").join("proj"));
        assert_eq!(parsed.args, vec!["ni", "vite"]);
    }
}
