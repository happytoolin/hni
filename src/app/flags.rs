use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct ParsedFlags {
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub debug: bool,
    pub show_help: bool,
    pub show_version: bool,
}

pub fn parse_global_flags(base_cwd: &Path, args: Vec<String>) -> Result<ParsedFlags> {
    let mut cwd = base_cwd.to_path_buf();
    let mut debug = false;
    let mut rest = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "?" => {
                debug = true;
                i += 1;
            }
            "-C" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| anyhow!("-C requires a path argument"))?;
                cwd = cwd.join(next);
                i += 2;
            }
            _ => {
                rest.push(args[i].clone());
                i += 1;
            }
        }
    }

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
}
