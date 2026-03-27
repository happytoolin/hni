use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::core::types::NativeLocalBinLauncher;

use super::shim_parser::{NodeBinLaunch, node_args_from_shebang, parse_node_shell_shim};

pub(super) fn resolve_local_bin_launcher(bin_path: &Path) -> Result<NativeLocalBinLauncher> {
    let inspected_path = resolve_bin_source_path(bin_path)?;

    if let Some(node_launch) = detect_node_launcher(&inspected_path)? {
        return Ok(NativeLocalBinLauncher::NodeScript {
            script_path: node_launch.script_path,
            node_args: node_launch.node_args,
        });
    }

    let extension = inspected_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    Ok(match extension.as_deref() {
        Some("cmd") | Some("bat") => NativeLocalBinLauncher::Cmd(inspected_path),
        Some("ps1") => NativeLocalBinLauncher::PowerShell(inspected_path),
        _ => NativeLocalBinLauncher::Binary(inspected_path),
    })
}

fn detect_node_launcher(inspected_path: &Path) -> Result<Option<NodeBinLaunch>> {
    if matches!(
        inspected_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("js") | Some("cjs") | Some("mjs")
    ) {
        return Ok(Some(NodeBinLaunch {
            script_path: inspected_path.to_path_buf(),
            node_args: Vec::new(),
        }));
    }

    let raw = match fs::read_to_string(inspected_path) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };

    if let Some(node_args) = raw.lines().next().and_then(node_args_from_shebang) {
        return Ok(Some(NodeBinLaunch {
            script_path: inspected_path.to_path_buf(),
            node_args,
        }));
    }

    Ok(parse_node_shell_shim(&raw, inspected_path))
}

fn resolve_bin_source_path(bin_path: &Path) -> Result<PathBuf> {
    let mut current = bin_path.to_path_buf();

    for _ in 0..8 {
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(_) => return Ok(current),
        };

        if !metadata.file_type().is_symlink() {
            return Ok(dunce::canonicalize(&current).unwrap_or(current));
        }

        let target = fs::read_link(&current)?;
        current = if target.is_absolute() {
            target
        } else {
            current
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(target)
        };
    }

    Ok(dunce::canonicalize(&current).unwrap_or(current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::NativeLocalBinLauncher;
    use std::fs;

    #[cfg(unix)]
    #[test]
    fn resolves_symlinked_js_bins_to_node_script_launcher() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let package_dir = dir.path().join("node_modules").join("tool");
        let bin_dir = dir.path().join("node_modules").join(".bin");
        fs::create_dir_all(&package_dir).unwrap();
        fs::create_dir_all(&bin_dir).unwrap();
        let script = package_dir.join("cli.js");
        fs::write(&script, "console.log('hi')").unwrap();
        let shim = bin_dir.join("tool");
        symlink("../tool/cli.js", &shim).unwrap();

        let launcher = resolve_local_bin_launcher(&shim).unwrap();
        assert_eq!(
            launcher,
            NativeLocalBinLauncher::NodeScript {
                script_path: dunce::canonicalize(&script).unwrap(),
                node_args: Vec::new(),
            }
        );
    }
}
