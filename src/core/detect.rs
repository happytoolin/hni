use std::{
    env,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};
use semver::Version;

use super::{
    config::{DefaultAgent, HniConfig},
    pkg_json::read_package_json,
    types::{DetectionResult, DetectionSource, PackageManager},
};

const LOCKFILES: &[(&str, PackageManager)] = &[
    ("bun.lockb", PackageManager::Bun),
    ("bun.lock", PackageManager::Bun),
    ("pnpm-lock.yaml", PackageManager::Pnpm),
    ("yarn.lock", PackageManager::Yarn),
    ("package-lock.json", PackageManager::Npm),
    ("npm-shrinkwrap.json", PackageManager::Npm),
    ("deno.lock", PackageManager::Deno),
    ("deno.json", PackageManager::Deno),
    ("deno.jsonc", PackageManager::Deno),
];

pub fn detect(cwd: &Path, config: &HniConfig) -> Result<DetectionResult> {
    let ancestors = cwd.ancestors().collect::<Vec<_>>();
    let has_lock = ancestors
        .iter()
        .any(|dir| detect_lockfile_in_dir(dir).is_some());

    for dir in ancestors {
        let package_manager_hint = read_package_json(dir)?
            .and_then(|package_json| package_json.package_manager)
            .and_then(|raw| parse_package_manager_field(&raw));

        if let Some((pm, version_hint)) = package_manager_hint {
            return Ok(DetectionResult {
                agent: Some(pm),
                has_lock,
                version_hint,
                source: DetectionSource::PackageManagerField,
            });
        }

        if let Some(pm) = detect_lockfile_in_dir(dir) {
            return Ok(DetectionResult {
                agent: Some(pm),
                has_lock,
                version_hint: None,
                source: DetectionSource::Lockfile,
            });
        }
    }

    if let DefaultAgent::Agent(agent) = config.default_agent {
        return Ok(DetectionResult {
            agent: Some(agent),
            has_lock,
            version_hint: None,
            source: DetectionSource::Config,
        });
    }

    if which::which("npm").is_ok() {
        return Ok(DetectionResult {
            agent: Some(PackageManager::Npm),
            has_lock,
            version_hint: None,
            source: DetectionSource::Fallback,
        });
    }

    Ok(DetectionResult {
        agent: None,
        has_lock,
        version_hint: None,
        source: DetectionSource::None,
    })
}

fn detect_lockfile_in_dir(dir: &Path) -> Option<PackageManager> {
    LOCKFILES
        .iter()
        .find_map(|(lockfile, pm)| dir.join(lockfile).exists().then_some(*pm))
}

pub fn ensure_package_manager_available(
    pm: PackageManager,
    version_hint: Option<&str>,
    config: &HniConfig,
    cwd: &Path,
) -> Result<()> {
    if env::var_os("HNI_SKIP_PM_CHECK").is_some() {
        return Ok(());
    }

    if which::which(pm.bin()).is_ok() {
        return Ok(());
    }

    if !config.auto_install {
        return Err(anyhow!(
            "detected {} but it is not installed. Set HNI_AUTO_INSTALL=true or install it manually",
            pm.display_name()
        ));
    }

    if env::var_os("CI").is_some() {
        eprintln!("[hni] auto-installing {} in CI mode", pm.display_name());
    }

    let package = pm.global_package_name();
    if package == "npm" {
        return Err(anyhow!(
            "npm is required for auto-install but was not found in PATH"
        ));
    }

    if matches!(pm, PackageManager::Deno) {
        return Err(anyhow!(
            "auto-install for deno is not supported; install deno manually"
        ));
    }

    if which::which("npm").is_err() {
        return Err(anyhow!(
            "auto-install requires npm in PATH, but npm is unavailable"
        ));
    }

    let target = match version_hint {
        Some(version) if !version.is_empty() => format!("{package}@{version}"),
        _ => package.to_string(),
    };

    let status = Command::new("npm")
        .args(["i", "-g", &target])
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run npm for auto-install")?;

    if !status.success() {
        return Err(anyhow!(
            "auto-install failed for {} with exit code {:?}",
            pm.display_name(),
            status.code()
        ));
    }

    if which::which(pm.bin()).is_err() {
        return Err(anyhow!(
            "auto-install for {} completed but binary is still not in PATH",
            pm.display_name()
        ));
    }

    Ok(())
}

fn parse_package_manager_field(value: &str) -> Option<(PackageManager, Option<String>)> {
    let (name, version) = value.split_once('@')?;
    let lower = name.to_ascii_lowercase();

    let mut pm = PackageManager::from_name(&lower)?;
    if pm == PackageManager::Yarn && parse_major(version).is_some_and(|major| major >= 2) {
        pm = PackageManager::YarnBerry;
    }

    Some((pm, Some(version.to_string())))
}

fn parse_major(version: &str) -> Option<u64> {
    Version::parse(version)
        .map(|parsed| parsed.major)
        .ok()
        .or_else(|| {
            Version::parse(&format!("{version}.0.0"))
                .map(|parsed| parsed.major)
                .ok()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::HniConfig;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_package_manager_field_first() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();

        let out = detect(dir.path(), &HniConfig::default()).unwrap();
        assert_eq!(out.agent, Some(PackageManager::Pnpm));
        assert_eq!(out.source, DetectionSource::PackageManagerField);
    }

    #[test]
    fn detects_lockfile_priority() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("yarn.lock"), "x").unwrap();
        fs::write(dir.path().join("pnpm-lock.yaml"), "x").unwrap();

        let out = detect(dir.path(), &HniConfig::default()).unwrap();
        assert_eq!(out.agent, Some(PackageManager::Pnpm));
    }

    #[test]
    fn package_manager_field_yarn_berry() {
        let parsed = parse_package_manager_field("yarn@4.2.1").unwrap();
        assert_eq!(parsed.0, PackageManager::YarnBerry);
    }

    #[test]
    fn yarn_lock_without_package_manager_stays_yarn_classic() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("yarn.lock"), "lock").unwrap();
        fs::write(dir.path().join(".yarnrc.yml"), "nodeLinker: pnp\n").unwrap();

        let out = detect(dir.path(), &HniConfig::default()).unwrap();
        assert_eq!(out.agent, Some(PackageManager::Yarn));
    }

    #[test]
    fn detects_workspace_root_package_manager_from_subpackage() {
        let root = tempdir().unwrap();
        fs::write(
            root.path().join("package.json"),
            r#"{"packageManager":"pnpm@9.0.0","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(root.path().join("pnpm-lock.yaml"), "lock").unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("package.json"), r#"{"name":"app"}"#).unwrap();

        let out = detect(&pkg, &HniConfig::default()).unwrap();
        assert_eq!(out.agent, Some(PackageManager::Pnpm));
        assert_eq!(out.source, DetectionSource::PackageManagerField);
        assert!(out.has_lock);
    }

    #[test]
    fn detects_workspace_lockfile_from_subpackage() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("pnpm-lock.yaml"), "lock").unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("package.json"), r#"{"name":"app"}"#).unwrap();

        let out = detect(&pkg, &HniConfig::default()).unwrap();
        assert_eq!(out.agent, Some(PackageManager::Pnpm));
        assert_eq!(out.source, DetectionSource::Lockfile);
        assert!(out.has_lock);
    }

    #[test]
    fn prefers_subpackage_lockfile_over_parent_package_manager() {
        let root = tempdir().unwrap();
        fs::write(
            root.path().join("package.json"),
            r#"{"packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("package.json"), r#"{"name":"app"}"#).unwrap();
        fs::write(pkg.join("package-lock.json"), "lock").unwrap();

        let out = detect(&pkg, &HniConfig::default()).unwrap();
        assert_eq!(out.agent, Some(PackageManager::Npm));
        assert_eq!(out.source, DetectionSource::Lockfile);
        assert!(out.has_lock);
    }
}
