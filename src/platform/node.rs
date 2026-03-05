use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

pub const REAL_NODE_ENV: &str = "HNI_REAL_NODE";
pub const SHIM_ACTIVE_ENV: &str = "HNI_NODE_SHIM_ACTIVE";

pub fn resolve_real_node_path() -> Result<PathBuf> {
    if let Some(from_env) = env::var_os(REAL_NODE_ENV) {
        let path = PathBuf::from(from_env);
        if path.exists() {
            return Ok(path);
        }
    }

    if let Some(recorded) = read_recorded_real_node_path()? {
        if recorded.exists() {
            return Ok(recorded);
        }
    }

    scan_path_for_real_node().ok_or_else(|| {
        anyhow!(
            "unable to locate real node binary. Set {}=/absolute/path/to/node",
            REAL_NODE_ENV
        )
    })
}

pub fn recorded_real_node_path_file() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("hni").join("real-node-path"))
}

fn read_recorded_real_node_path() -> Result<Option<PathBuf>> {
    let Some(path) = recorded_real_node_path_file() else {
        return Ok(None);
    };

    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(path)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(PathBuf::from(trimmed)))
}

fn scan_path_for_real_node() -> Option<PathBuf> {
    let current_exe = env::current_exe().ok();
    let current_dir = current_exe
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let candidates = which::which_all("node").ok()?;
    for candidate in candidates {
        if let Some(current_dir) = &current_dir {
            if let Some(parent) = candidate.parent() {
                if paths_equal(parent, current_dir) {
                    continue;
                }
            }
        }

        if let Some(current_exe) = &current_exe {
            if paths_equal(&candidate, current_exe) {
                continue;
            }
        }
        return Some(candidate);
    }

    None
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}
