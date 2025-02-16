use anyhow::Result;
use config::{Config, Environment, File};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, env, fs, path::Path};

use crate::{
    package_managers::{
        bun::BunFactory, deno::DenoFactory, npm::NpmFactory, pnpm::PnpmFactory,
        pnpm6::Pnpm6Factory, yarn::YarnFactory, yarn_berry::YarnBerryFactory,
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
            PackageManagerFactoryEnum::Yarn => Box::new(YarnFactory {}),
            PackageManagerFactoryEnum::Pnpm => Box::new(PnpmFactory {}),
            PackageManagerFactoryEnum::Bun => Box::new(BunFactory {}),
            PackageManagerFactoryEnum::Deno => Box::new(DenoFactory {}),
            PackageManagerFactoryEnum::YarnBerry => Box::new(YarnBerryFactory {}),
            PackageManagerFactoryEnum::Pnpm6 => Box::new(Pnpm6Factory {}),
        }
    }
}

// the order here matters, more specific one comes first
pub fn get_locks() -> HashMap<&'static str, PackageManagerFactoryEnum> {
    trace!("Initializing package manager lockfile mapping");
    let mut locks = HashMap::new();
    locks.insert("bun.lock", PackageManagerFactoryEnum::Bun);
    locks.insert("bun.lockb", PackageManagerFactoryEnum::Bun);
    locks.insert("pnpm-lock.yaml", PackageManagerFactoryEnum::Pnpm);
    locks.insert("yarn.lock", PackageManagerFactoryEnum::Yarn);
    locks.insert("package-lock.json", PackageManagerFactoryEnum::Npm);
    locks.insert("npm-shrinkwrap.json", PackageManagerFactoryEnum::Npm);
    trace!("Registered {} lockfile patterns", locks.len());
    locks
}

#[derive(Debug, Deserialize)]
struct NirsConfig {
    #[serde(default = "default_package_manager")]
    default_package_manager: Option<String>,
}

fn default_package_manager() -> Option<String> {
    Some("npm".to_string())
}

pub fn detect(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    info!("Detecting package manager in directory: {}", cwd.display());

    if !cwd.exists() {
        warn!("Directory does not exist: {}", cwd.display());
        return Ok(None);
    }

    // Check packageManager field in package.json
    debug!("Checking packageManager in package.json");
    let package_json_path = cwd.join("package.json");
    if package_json_path.exists() {
        if let Ok(contents) = fs::read_to_string(package_json_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                if let Some(package_manager) = json.get("packageManager") {
                    if let Some(pm) = package_manager.as_str() {
                        info!("Found packageManager in package.json: {}", pm);
                        match pm.split("@").next().unwrap() {
                            "npm" => return Ok(Some(PackageManagerFactoryEnum::Npm)),
                            "yarn" => return Ok(Some(PackageManagerFactoryEnum::Yarn)),
                            "pnpm" => return Ok(Some(PackageManagerFactoryEnum::Pnpm)),
                            "bun" => return Ok(Some(PackageManagerFactoryEnum::Bun)),
                            _ => warn!("Unknown package manager: {}", pm),
                        }
                    } else {
                        warn!("packageManager field is not a string");
                    }
                }
            } else {
                warn!("Failed to parse package.json");
            }
        } else {
            warn!("Failed to read package.json");
        }
    }

    // Check for lockfiles
    let locks = get_locks();
    debug!("Checking for {} known lockfile patterns", locks.len());

    for (lock, package_manager) in locks.iter() {
        let lockfile = cwd.join(lock);
        trace!("Checking for lockfile: {}", lockfile.display());

        if lockfile.exists() {
            info!(
                "Found package manager {} (lockfile: {})",
                package_manager, lock
            );
            debug!(
                "Using package manager {} based on lockfile {}",
                package_manager, lock
            );
            return Ok(Some(*package_manager));
        }
    }

    // Check for config file
    debug!("No lockfile or packageManager found, checking config");
    let config = Config::builder()
        .add_source(File::with_name("nirs.toml").required(false))
        .add_source(File::with_name("nirs.json").required(false))
        .add_source(File::with_name("nirs.yaml").required(false))
        .add_source(File::with_name(&format!("{}/nirs.toml", env!("HOME"))).required(false))
        .add_source(File::with_name(&format!("{}/nirs.json", env!("HOME"))).required(false))
        .add_source(File::with_name(&format!("{}/nirs.yaml", env!("HOME"))).required(false))
        .add_source(Environment::with_prefix("NIRS"))
        .build()?;

    let nirs_config: NirsConfig = match config.try_deserialize() {
        Ok(c) => {
            info!("Config loaded successfully: {:?}", c);
            c
        }
        Err(e) => {
            warn!("Failed to load config: {}", e);
            NirsConfig {
                default_package_manager: None,
            }
        }
    };

    debug!(
        "Default package manager from config: {:?}",
        nirs_config.default_package_manager
    );

    if let Some(pm) = nirs_config.default_package_manager {
        match pm.as_str() {
            "npm" => {
                info!("No lockfile or packageManager found, defaulting to npm from config");
                return Ok(Some(PackageManagerFactoryEnum::Npm));
            }
            "yarn" => {
                info!("No lockfile or packageManager found, defaulting to yarn from config");
                return Ok(Some(PackageManagerFactoryEnum::Yarn));
            }
            "pnpm" => {
                info!("No lockfile or packageManager found, defaulting to pnpm from config");
                return Ok(Some(PackageManagerFactoryEnum::Pnpm));
            }
            "bun" => {
                info!("No lockfile or packageManager found, defaulting to bun from config");
                return Ok(Some(PackageManagerFactoryEnum::Bun));
            }
            _ => {
                warn!("Invalid default package manager in config: {}", pm);
                if let Ok(path) = env::var("PATH") {
                    trace!("PATH environment variable: {}", path);
                    if path.contains("npm") {
                        info!("No lockfile or packageManager found, defaulting to npm from config");
                        debug!("Using npm as fallback package manager");
                        return Ok(Some(PackageManagerFactoryEnum::Npm));
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
        }
    }

    // Fallback to npm if in PATH
    if let Ok(path) = env::var("PATH") {
        trace!("PATH environment variable: {}", path);
        if path.contains("npm") {
            info!("No lockfile or packageManager found, defaulting to npm (found in PATH)");
            debug!("Using npm as fallback package manager");
            return Ok(Some(PackageManagerFactoryEnum::Npm));
        }
    }

    warn!("No package manager detected in {}", cwd.display());
    return Ok(None);
}

pub fn detect_sync(cwd: &Path) -> Option<PackageManagerFactoryEnum> {
    trace!("Running synchronous package manager detection");

    // Check packageManager field in package.json
    debug!("Checking packageManager in package.json");
    let package_json_path = cwd.join("package.json");
    if package_json_path.exists() {
        if let Ok(contents) = fs::read_to_string(package_json_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                if let Some(package_manager) = json.get("packageManager") {
                    if let Some(pm) = package_manager.as_str() {
                        info!("Found packageManager in package.json: {}", pm);
                        match pm.split("@").next().unwrap() {
                            "npm" => return Some(PackageManagerFactoryEnum::Npm),
                            "yarn" => return Some(PackageManagerFactoryEnum::Yarn),
                            "pnpm" => return Some(PackageManagerFactoryEnum::Pnpm),
                            "bun" => return Some(PackageManagerFactoryEnum::Bun),
                            _ => warn!("Unknown package manager: {}", pm),
                        }
                    } else {
                        warn!("packageManager field is not a string");
                    }
                }
            }
        }
    }

    // Fallback to checking for npm if no lockfile is found
    if env::var("PATH")
        .ok()
        .is_some_and(|path| path.contains("npm"))
    {
        debug!("Sync detection defaulting to npm");
        return Some(PackageManagerFactoryEnum::Npm);
    }

    debug!("Sync detection found no package manager");
    None
}
