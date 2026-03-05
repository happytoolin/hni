use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct PackageJson {
    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,
    pub scripts: Option<BTreeMap<String, String>>,
    #[serde(rename = "scripts-info")]
    pub scripts_info: Option<BTreeMap<String, String>>,
    pub dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "optionalDependencies")]
    pub optional_dependencies: Option<BTreeMap<String, String>>,
}

pub fn package_json_path(cwd: &Path) -> PathBuf {
    cwd.join("package.json")
}

pub fn read_package_json(cwd: &Path) -> Result<Option<PackageJson>> {
    let path = package_json_path(cwd);
    let raw = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display()))?,
    };

    let parsed: PackageJson = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(parsed))
}
