use std::{collections::HashMap, env, path::Path};
use log::{info, debug};

use crate::{
    package_managers::{
        bun::BunFactory, deno::DenoFactory, npm::NpmFactory, pnpm::PnpmFactory,
        pnpm6::Pnpm6Factory, yarn::YarnFactory, yarn_berry::YarnBerryFactory,
    },
    PackageManagerFactory,
};

use anyhow::Result;

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
    let mut locks = HashMap::new();
    locks.insert("bun.lock", PackageManagerFactoryEnum::Bun);
    locks.insert("bun.lockb", PackageManagerFactoryEnum::Bun);
    locks.insert("pnpm-lock.yaml", PackageManagerFactoryEnum::Pnpm);
    locks.insert("yarn.lock", PackageManagerFactoryEnum::Yarn);
    locks.insert("package-lock.json", PackageManagerFactoryEnum::Npm);
    locks.insert("npm-shrinkwrap.json", PackageManagerFactoryEnum::Npm);
    locks
}

pub fn detect(cwd: &Path) -> Result<Option<PackageManagerFactoryEnum>> {
    info!("Detecting package manager in directory: {}", cwd.display());
    let locks = get_locks();
    
    for (lock, package_manager) in locks.iter() {
        debug!("Checking for lockfile: {}", lock);
        if cwd.join(lock).exists() {
            info!("Found package manager {} (lockfile: {})", package_manager, lock);
            return Ok(Some(*package_manager));
        }
    }

    // Fallback to checking for npm if no lockfile is found
    if let Ok(path) = env::var("PATH") {
        if path.contains("npm") {
            info!("No lockfile found, defaulting to npm (found in PATH)");
            return Ok(Some(PackageManagerFactoryEnum::Npm));
        }
    }

    info!("No package manager detected");
    Ok(None)
}

pub fn detect_sync(cwd: &Path) -> Option<PackageManagerFactoryEnum> {
    let locks = get_locks();
    for (lock, package_manager) in locks.iter() {
        if cwd.join(lock).exists() {
            return Some(*package_manager);
        }
    }

    // Fallback to checking for npm if no lockfile is found
    if env::var("PATH")
        .ok()
        .is_some_and(|path| path.contains("npm"))
    {
        return Some(PackageManagerFactoryEnum::Npm);
    }

    None
}
