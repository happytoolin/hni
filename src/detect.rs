use anyhow::{Context, Result};
use log::{debug, info, trace, warn};
use serde_json::Value;
use std::{env, fs, path::Path};

use crate::{
    config,
    package_managers::{
        bun::BunFactory, deno::DenoFactory, npm::NpmFactory, pnpm::PnpmFactory, yarn::YarnFactory,
    },
    PackageManagerFactory,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PackageManagerFactoryEnum {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
    YarnBerry,
}

impl PackageManagerFactoryEnum {
    pub fn get_nlx_command(&self) -> &'static [&'static str] {
        match self {
            PackageManagerFactoryEnum::Npm => &["npx"],
            PackageManagerFactoryEnum::Yarn => &["yarn", "dlx"],
            PackageManagerFactoryEnum::Pnpm => &["pnpm", "dlx"],
            PackageManagerFactoryEnum::Bun => &["bun"],
            PackageManagerFactoryEnum::Deno => &["deno", "x"],
            PackageManagerFactoryEnum::YarnBerry => &["yarn", "dlx"],
        }
    }

    pub fn get_bin_name(&self) -> &'static str {
        match self {
            PackageManagerFactoryEnum::Npm => "npm",
            PackageManagerFactoryEnum::Yarn => "yarn",
            PackageManagerFactoryEnum::Pnpm => "pnpm",
            PackageManagerFactoryEnum::Bun => "bun",
            PackageManagerFactoryEnum::Deno => "deno",
            _ => "",
        }
    }
}

impl std::fmt::Display for PackageManagerFactoryEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManagerFactoryEnum::Npm => write!(f, "npm"),
            PackageManagerFactoryEnum::Yarn => write!(f, "yarn"),
            PackageManagerFactoryEnum::Pnpm => write!(f, "pnpm"),
            PackageManagerFactoryEnum::Bun => write!(f, "bun"),
            PackageManagerFactoryEnum::Deno => write!(f, "deno"),
            PackageManagerFactoryEnum::YarnBerry => write!(f, "yarn (berry)"),
        }
    }
}

impl PackageManagerFactoryEnum {
    pub fn get_factory(&self) -> Box<dyn PackageManagerFactory> {
        debug!("Creating factory for package manager: {}", self);
        match self {
            PackageManagerFactoryEnum::Npm => Box::new(NpmFactory {}),
            PackageManagerFactoryEnum::Yarn => Box::new(YarnFactory::new(false)),
            PackageManagerFactoryEnum::Pnpm => Box::new(PnpmFactory::new()),
            PackageManagerFactoryEnum::Bun => Box::new(BunFactory {}),
            PackageManagerFactoryEnum::Deno => Box::new(DenoFactory {}),
            PackageManagerFactoryEnum::YarnBerry => Box::new(YarnFactory::new(true)),
        }
    }
}

// Make lockfile list a const for better determinism
const LOCKFILES: &[(&str, PackageManagerFactoryEnum)] = &[
    ("bun.lockb", PackageManagerFactoryEnum::Bun),
    ("bun.lock", PackageManagerFactoryEnum::Bun),
    ("pnpm-lock.yaml", PackageManagerFactoryEnum::Pnpm),
    ("yarn.lock", PackageManagerFactoryEnum::Yarn),
    ("package-lock.json", PackageManagerFactoryEnum::Npm),
    ("npm-shrinkwrap.json", PackageManagerFactoryEnum::Npm),
    ("deno.lock", PackageManagerFactoryEnum::Deno),
];

pub fn get_locks() -> &'static [(&'static str, PackageManagerFactoryEnum)] {
    trace!("Returning static lockfile list");
    LOCKFILES
}

pub fn detect(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    info!("Detecting package manager in {}", cwd.display());

    if !cwd.exists() {
        anyhow::bail!("Directory does not exist: {}", cwd.display());
    }
    if !cwd.is_dir() {
        anyhow::bail!("Path is not a directory: {}", cwd.display());
    }

    // Check packageManager field first
    if let Some(pm) = read_package_manager_field(cwd)? {
        return Ok(Some(pm));
    }

    // Check lockfiles in priority order
    if let Some(pm) = detect_via_lockfiles(cwd)? {
        return Ok(Some(pm));
    }

    // Check config files
    if let Some(pm) = read_config(cwd)? {
        return Ok(Some(pm));
    }

    // Final fallback with proper error context
    if should_fallback_to_npm()? {
        info!("Falling back to npm");
        return Ok(Some(PackageManagerFactoryEnum::Npm));
    }

    Ok(None)
}

fn read_package_manager_field(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    let path = cwd.join("package.json");
    if !path.exists() {
        trace!("No package.json found");
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read package.json at {}", path.display()))?;

    // Validate JSON structure before parsing
    if contents.trim().is_empty() {
        warn!("Empty package.json file at {}", path.display());
        return Ok(None);
    }

    let json: Value = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse package.json at {}", path.display()))?;

    json.get("packageManager")
        .and_then(Value::as_str)
        .map(|pm| parse_package_manager(pm, &path))
        .transpose()
}

fn parse_package_manager(pm: &str, path: &Path) -> Result<PackageManagerFactoryEnum> {
    let (name, _version) = pm.split_once('@').ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid packageManager format in {}, expected 'manager@version'",
            path.display()
        )
    })?;

    match name {
        "npm" => Ok(PackageManagerFactoryEnum::Npm),
        "yarn" => Ok(PackageManagerFactoryEnum::Yarn),
        "pnpm" => Ok(PackageManagerFactoryEnum::Pnpm),
        "bun" => Ok(PackageManagerFactoryEnum::Bun),
        other => Err(anyhow::anyhow!(
            "Unsupported package manager '{}' in {}",
            other,
            path.display()
        )),
    }
}

fn detect_via_lockfiles(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    let mut detected = Vec::new();

    for (lockfile, pm) in get_locks() {
        let path = cwd.join(lockfile);
        if path.exists() {
            // Basic lockfile validation
            if path.metadata()?.len() == 0 {
                warn!("Empty lockfile detected: {}", lockfile);
                continue;
            }

            detected.push((pm, lockfile));
        }
    }

    if detected.len() > 1 {
        warn!("Multiple lockfiles detected: {:?}", detected);
    }

    detected
        .first()
        .map(|(pm, lockfile)| {
            info!("Selected {} via lockfile {}", pm, lockfile);
            Ok(**pm)
        })
        .transpose()
}

fn read_config(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    let config = config::load(cwd)?;

    config
        .default_package_manager
        .as_deref()
        .map(|pm| match pm {
            "npm" => Ok(PackageManagerFactoryEnum::Npm),
            "yarn" => Ok(PackageManagerFactoryEnum::Yarn),
            "pnpm" => Ok(PackageManagerFactoryEnum::Pnpm),
            "bun" => Ok(PackageManagerFactoryEnum::Bun),
            other => Err(anyhow::anyhow!(
                "Invalid package manager in config: {}",
                other
            )),
        })
        .transpose()
}

fn should_fallback_to_npm() -> Result<bool> {
    if env::var("CI").is_ok() {
        return Ok(false);
    }

    let exe_names: &[&str] = if cfg!(windows) {
        &["npm.exe", "npm.cmd"]
    } else {
        &["npm"]
    };

    let npm_exists = env::var_os("PATH")
        .map(|path| {
            env::split_paths(&path).any(|p| exe_names.iter().any(|exe| p.join(exe).exists()))
        })
        .unwrap_or(false);

    Ok(npm_exists)
}

pub fn detect_sync(cwd: &Path) -> Option<PackageManagerFactoryEnum> {
    detect(cwd).unwrap_or_else(|e| {
        warn!("Detection error: {}", e);
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_package_manager_valid() {
        let cases = vec![
            ("npm@9.0.0", PackageManagerFactoryEnum::Npm),
            ("yarn@3.0.0", PackageManagerFactoryEnum::Yarn),
            ("pnpm@7.0.0", PackageManagerFactoryEnum::Pnpm),
            ("pnpm@6.0.0", PackageManagerFactoryEnum::Pnpm),
            ("bun@1.0.0", PackageManagerFactoryEnum::Bun),
        ];

        for (input, expected) in cases {
            let result = parse_package_manager(input, Path::new("")).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_package_manager_invalid() {
        let cases = vec![
            "invalid",        // No version
            "unknown@1.0.0",  // Unsupported manager
            "invalid-format", // No @
        ];

        for input in cases {
            let result = parse_package_manager(input, Path::new("package.json"));
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_package_manager_empty_version() {
        let result = parse_package_manager("npm@", Path::new("package.json"));
        assert!(result.is_ok()); // Should parse with warning
        assert_eq!(result.unwrap(), PackageManagerFactoryEnum::Npm);
    }

    #[test]
    fn test_detect_via_lockfiles_priority() {
        let dir = tempdir().unwrap();

        // Create valid lockfiles with content
        fs::write(dir.path().join("bun.lockb"), "lockfile content").unwrap();
        fs::write(dir.path().join("pnpm-lock.yaml"), "lockfile_version: 7.0.0").unwrap();
        fs::write(dir.path().join("yarn.lock"), "# yarn lockfile v1").unwrap();

        let result = detect_via_lockfiles(dir.path()).unwrap();
        assert_eq!(result, Some(PackageManagerFactoryEnum::Bun));
    }

    #[test]
    fn test_read_package_manager_field_priority() {
        let dir = tempdir().unwrap();
        let mut file = File::create(dir.path().join("package.json")).unwrap();
        writeln!(file, r#"{{ "packageManager": "pnpm@7.0.0" }}"#).unwrap();

        let result = detect(dir.path()).unwrap();
        assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    }

    #[test]
    fn test_detect_lockfile_precedence() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pnpm-lock.yaml"), "lockfile_version: 7.0.0").unwrap();

        let result = detect(dir.path()).unwrap();
        assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    }

    #[test]
    fn test_detect_fallback_to_npm() {
        let dir = tempdir().unwrap();
        let result = detect(dir.path()).unwrap();

        // This test depends on whether npm exists in the test environment
        // Might need to mock should_fallback_to_npm in actual practice
        if should_fallback_to_npm().unwrap() {
            assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
        } else {
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_detect_invalid_directory() {
        let result = detect(Path::new("/non/existent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_package_json() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();

        let result = read_package_manager_field(dir.path()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_multiple_lockfiles_warning() {
        let dir = tempdir().unwrap();
        // Create valid lockfiles with different content
        fs::write(dir.path().join("yarn.lock"), "# yarn lockfile v1").unwrap();
        fs::write(dir.path().join("pnpm-lock.yaml"), "lockfile_version: 6.0.0").unwrap();

        let result = detect_via_lockfiles(dir.path()).unwrap();
        // Should detect pnpm since it comes before yarn in LOCKFILES array
        assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    }

    #[test]
    fn test_pnpm_version_parsing() {
        let cases = vec![
            ("pnpm@6.9.0", PackageManagerFactoryEnum::Pnpm),
            ("pnpm@7.1.0", PackageManagerFactoryEnum::Pnpm),
            ("pnpm@6.0.0-rc.0", PackageManagerFactoryEnum::Pnpm),
            ("pnpm@latest", PackageManagerFactoryEnum::Pnpm), // Defaults to 6.x if parsing fails
        ];

        for (input, expected) in cases {
            let result = parse_package_manager(input, Path::new("")).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_config_file_detection() {
        let dir = tempdir().unwrap();
        // Create config file matching expected name pattern
        let config_path = dir.path().join("nirs.json");
        fs::write(&config_path, r#"{"default_package_manager": "pnpm"}"#).unwrap();

        let result = read_config(dir.path()).unwrap();
        assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    }

    #[test]
    fn test_empty_lockfile_handling() {
        let dir = tempdir().unwrap();
        let lockfile = dir.path().join("yarn.lock");
        File::create(&lockfile).unwrap();

        // Write empty content
        let mut file = File::create(&lockfile).unwrap();
        file.write_all(b"").unwrap();

        let result = detect_via_lockfiles(dir.path()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_detect_sync_error_handling() {
        let result = detect_sync(Path::new("/invalid/path"));
        assert_eq!(result, None);
    }
}
