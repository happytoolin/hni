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
    Pnpm6,
}

impl PackageManagerFactoryEnum {
    pub fn get_nlx_command(&self) -> &'static [&'static str] {
        match self {
            PackageManagerFactoryEnum::Npm => &["npx"],
            PackageManagerFactoryEnum::Yarn => &["yarn", "dlx"],
            PackageManagerFactoryEnum::Pnpm => &["pnpm"],
            PackageManagerFactoryEnum::Bun => &["bun"],
            PackageManagerFactoryEnum::Deno => &["deno", "x"],
            PackageManagerFactoryEnum::YarnBerry => &["yarn", "dlx"],
            PackageManagerFactoryEnum::Pnpm6 => &["pnpm", "dlx"],
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
            PackageManagerFactoryEnum::Pnpm6 => write!(f, "pnpm6"),
        }
    }
}

impl PackageManagerFactoryEnum {
    pub fn get_factory(&self) -> Box<dyn PackageManagerFactory> {
        debug!("Creating factory for package manager: {}", self);
        match self {
            PackageManagerFactoryEnum::Npm => Box::new(NpmFactory {}),
            PackageManagerFactoryEnum::Yarn => Box::new(YarnFactory::new(false)),
            PackageManagerFactoryEnum::Pnpm => Box::new(PnpmFactory::new(false)),
            PackageManagerFactoryEnum::Bun => Box::new(BunFactory {}),
            PackageManagerFactoryEnum::Deno => Box::new(DenoFactory {}),
            PackageManagerFactoryEnum::YarnBerry => Box::new(YarnFactory::new(true)),
            PackageManagerFactoryEnum::Pnpm6 => Box::new(PnpmFactory::new(true)),
        }
    }
}

// Make lockfile list a const for better determinism
const LOCKFILES: &[(&str, PackageManagerFactoryEnum)] = &[
    ("bun.lockb", PackageManagerFactoryEnum::Bun),
    ("bun.lock", PackageManagerFactoryEnum::Bun),
    ("pnpm-lock.yaml", PackageManagerFactoryEnum::Pnpm),
    ("pnpm-lock.yaml", PackageManagerFactoryEnum::Pnpm6),
    ("yarn.lock", PackageManagerFactoryEnum::Yarn),
    ("package-lock.json", PackageManagerFactoryEnum::Npm),
    ("npm-shrinkwrap.json", PackageManagerFactoryEnum::Npm),
];

pub fn get_locks() -> &'static [(&'static str, PackageManagerFactoryEnum)] {
    trace!("Returning static lockfile list");
    LOCKFILES
}

// Improved error handling with context
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

// Extract package.json parsing to separate function
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

// Improved parsing with split_once
fn parse_package_manager(pm: &str, path: &Path) -> Result<PackageManagerFactoryEnum> {
    let (name, version) = pm.split_once('@').ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid packageManager format in {}, expected 'manager@version'",
            path.display()
        )
    })?;

    if version.is_empty() {
        warn!(
            "Missing version in packageManager field at {}",
            path.display()
        );
    }

    match name {
        "npm" => Ok(PackageManagerFactoryEnum::Npm),
        "yarn" => Ok(PackageManagerFactoryEnum::Yarn),
        "pnpm" => {
            let version = version.parse::<f32>().unwrap_or(0.0);
            Ok(if version >= 7.0 {
                PackageManagerFactoryEnum::Pnpm
            } else {
                PackageManagerFactoryEnum::Pnpm6
            })
        }
        "bun" => Ok(PackageManagerFactoryEnum::Bun),
        other => Err(anyhow::anyhow!(
            "Unsupported package manager '{}' in {}",
            other,
            path.display()
        )),
    }
}

// Separate lockfile detection logic
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

// Update the read_config function
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

// Extract npm fallback logic
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

// Unified sync detection using same logic
pub fn detect_sync(cwd: &Path) -> Option<PackageManagerFactoryEnum> {
    detect(cwd).unwrap_or_else(|e| {
        warn!("Detection error: {}", e);
        None
    })
}
