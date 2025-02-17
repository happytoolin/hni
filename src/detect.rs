use anyhow::Result;
use config::{Config, Environment, File};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use serde_json::Value;
use std::{env, fs, path::Path};

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

impl PackageManagerFactoryEnum {
    pub fn get_nlx_command(&self) -> Vec<&str> {
        match self {
            PackageManagerFactoryEnum::Npm => vec!["npx"],
            PackageManagerFactoryEnum::Yarn => vec!["yarn", "dlx"],
            PackageManagerFactoryEnum::Pnpm => vec!["pnpm"],
            PackageManagerFactoryEnum::Bun => vec!["bun"],
            PackageManagerFactoryEnum::Deno => vec!["deno", "x"],
            PackageManagerFactoryEnum::YarnBerry => vec!["yarn", "dlx"],
            PackageManagerFactoryEnum::Pnpm6 => vec!["pnpm", "dlx"],
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
            PackageManagerFactoryEnum::Yarn => Box::new(YarnFactory {}),
            PackageManagerFactoryEnum::Pnpm => Box::new(PnpmFactory {}),
            PackageManagerFactoryEnum::Bun => Box::new(BunFactory {}),
            PackageManagerFactoryEnum::Deno => Box::new(DenoFactory {}),
            PackageManagerFactoryEnum::YarnBerry => Box::new(YarnBerryFactory {}),
            PackageManagerFactoryEnum::Pnpm6 => Box::new(Pnpm6Factory {}),
        }
    }
}

// Change lock file storage to a Vec to maintain priority order
pub fn get_locks() -> Vec<(&'static str, PackageManagerFactoryEnum)> {
    trace!("Initializing package manager lockfile mapping");
    let locks = vec![
        ("bun.lockb", PackageManagerFactoryEnum::Bun),
        ("bun.lock", PackageManagerFactoryEnum::Bun),
        ("pnpm-lock.yaml", PackageManagerFactoryEnum::Pnpm),
        ("yarn.lock", PackageManagerFactoryEnum::Yarn),
        ("package-lock.json", PackageManagerFactoryEnum::Npm),
        ("npm-shrinkwrap.json", PackageManagerFactoryEnum::Npm),
    ];
    trace!("Registered {} lockfile patterns", locks.len());
    locks
}

#[derive(Debug, Deserialize)]
struct NirsConfig {
    #[serde(default)]
    default_package_manager: Option<String>,
}

pub fn detect(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    info!("Detecting package manager in directory: {}", cwd.display());

    if !cwd.exists() {
        warn!("Directory does not exist: {}", cwd.display());
        return Ok(None);
    }

    // Check CI environment first
    if env::var("CI").is_ok() {
        info!("CI environment detected, skipping PATH-based detection");
        return Ok(None);
    }

    // Check packageManager field in package.json
    debug!("Checking packageManager in package.json");
    let package_json_path = cwd.join("package.json");
    let package_json_result: Result<Option<Value>> = if package_json_path.exists() {
        match fs::read_to_string(package_json_path) {
            Ok(contents) => match serde_json::from_str::<Value>(&contents) {
                Ok(json) => Ok(Some(json)),
                Err(e) => {
                    warn!("Failed to parse package.json: {}", e);
                    Ok(None)
                }
            },
            Err(e) => {
                warn!("Failed to read package.json: {}", e);
                Ok(None)
            }
        }
    } else {
        Ok(None)
    };

    if let Ok(Some(json)) = package_json_result {
        if let Some(package_manager) = json.get("packageManager") {
            if let Some(pm) = package_manager.as_str() {
                info!("Found packageManager in package.json: {}", pm);
                match pm.split("@").next().unwrap() {
                    "npm" => return Ok(Some(PackageManagerFactoryEnum::Npm)),
                    "yarn" => return Ok(Some(PackageManagerFactoryEnum::Yarn)),
                    "pnpm" => return Ok(Some(PackageManagerFactoryEnum::Pnpm)),
                    "bun" => return Ok(Some(PackageManagerFactoryEnum::Bun)),
                    _ => {
                        warn!("Unknown package manager: {}", pm);
                        return Ok(None);
                    }
                }
            } else {
                warn!("packageManager field is not a string");
                return Ok(None);
            }
        }
    }

    // Check for lockfiles in priority order
    let locks = get_locks();
    debug!("Checking for {} known lockfile patterns", locks.len());

    if let Some((lock, package_manager)) = locks
        .iter()
        .find(|(lock_name, _)| cwd.join(lock_name).exists())
    {
        info!(
            "Found package manager {} (lockfile: {})",
            package_manager, lock
        );
        return Ok(Some(*package_manager));
    }

    // Check for config file
    debug!("No lockfile or packageManager found, checking config");
    let config = Config::builder()
        .add_source(File::from(cwd.join("nirs.toml")).required(false))
        .add_source(File::from(cwd.join("nirs.json")).required(false))
        .add_source(File::from(cwd.join("nirs.yaml")).required(false))
        .add_source(
            File::from(
                Path::new(&env::var("HOME").unwrap_or_default())
                    .join(".config")
                    .join("nirs.toml"),
            )
            .required(false),
        )
        .add_source(
            File::from(
                Path::new(&env::var("HOME").unwrap_or_default())
                    .join(".config")
                    .join("nirs.json"),
            )
            .required(false),
        )
        .add_source(
            File::from(
                Path::new(&env::var("HOME").unwrap_or_default())
                    .join(".config")
                    .join("nirs.yaml"),
            )
            .required(false),
        )
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
                info!("Using default package manager from config: npm");
                return Ok(Some(PackageManagerFactoryEnum::Npm));
            }
            "yarn" => {
                info!("Using default package manager from config: yarn");
                return Ok(Some(PackageManagerFactoryEnum::Yarn));
            }
            "pnpm" => {
                info!("Using default package manager from config: pnpm");
                return Ok(Some(PackageManagerFactoryEnum::Pnpm));
            }
            "bun" => {
                info!("Using default package manager from config: bun");
                return Ok(Some(PackageManagerFactoryEnum::Bun));
            }
            _ => {
                warn!("Invalid default package manager in config: {}", pm);
            }
        }
    }

    // Final fallback to npm if nothing else found
    if env::var("CI").is_err() {
        if let Ok(path) = env::var("PATH") {
            debug!("Checking PATH for npm: {}", path);
            let npm_exists = env::split_paths(&path)
                .inspect(|p| debug!("Checking directory: {}", p.display()))
                .any(|p| {
                    let npm_path = p.join(if cfg!(windows) { "npm.exe" } else { "npm" });
                    debug!("Checking for npm at: {}", npm_path.display());
                    let exists = npm_path.exists();
                    debug!(
                        "npm {} at {}",
                        if exists { "found" } else { "not found" },
                        npm_path.display()
                    );
                    exists
                });
            if npm_exists {
                info!("No package manager detected, falling back to npm");
                return Ok(Some(PackageManagerFactoryEnum::Npm));
            }
        }
    }

    warn!("No package manager detected in {}", cwd.display());
    Ok(None)
}

fn get_package_manager_from_package_json(cwd: &Path) -> Option<PackageManagerFactoryEnum> {
    let package_json_path = cwd.join("package.json");
    if let Ok(contents) = fs::read_to_string(package_json_path) {
        if let Ok(json) = serde_json::from_str::<Value>(&contents) {
            if let Some(package_manager) = json.get("packageManager") {
                if let Some(pm) = package_manager.as_str() {
                    info!("Found packageManager in package.json: {}", pm);
                    return match pm.split("@").next().unwrap() {
                        "npm" => Some(PackageManagerFactoryEnum::Npm),
                        "yarn" => Some(PackageManagerFactoryEnum::Yarn),
                        "pnpm" => Some(PackageManagerFactoryEnum::Pnpm),
                        "bun" => Some(PackageManagerFactoryEnum::Bun),
                        _ => {
                            warn!("Unknown package manager: {}", pm);
                            None
                        }
                    };
                } else {
                    warn!("packageManager field is not a string");
                }
            }
        }
    }
    None
}

pub fn detect_sync(cwd: &Path) -> Option<PackageManagerFactoryEnum> {
    trace!("Running synchronous package manager detection");

    // Check packageManager field in package.json
    debug!("Checking packageManager in package.json");
    if let Some(pm) = get_package_manager_from_package_json(cwd) {
        return Some(pm);
    }

    // Fallback to checking for npm if no lockfile is found
    if let Ok(path) = env::var("PATH") {
        debug!("Checking PATH for npm: {}", path);
        let npm_exists = env::split_paths(&path)
            .inspect(|p| debug!("Checking directory: {}", p.display()))
            .any(|p| {
                let npm_path = p.join(if cfg!(windows) { "npm.exe" } else { "npm" });
                debug!("Checking for npm at: {}", npm_path.display());
                let exists = npm_path.exists();
                debug!(
                    "npm {} at {}",
                    if exists { "found" } else { "not found" },
                    npm_path.display()
                );
                exists
            });
        if npm_exists {
            info!("No package manager detected, falling back to npm");
            return Some(PackageManagerFactoryEnum::Npm);
        }
    }

    debug!("Sync detection found no package manager");
    None
}
