use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
    YarnBerry,
    Pnpm6,
}

impl PackageManager {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
            PackageManager::Deno => "deno",
            PackageManager::YarnBerry => "yarn@berry",
            PackageManager::Pnpm6 => "pnpm@6",
        }
    }
}

// the order here matters, more specific one comes first
pub fn get_locks() -> HashMap<&'static str, PackageManager> {
    let mut locks = HashMap::new();
    locks.insert("bun.lock", PackageManager::Bun);
    locks.insert("bun.lockb", PackageManager::Bun);
    locks.insert("deno.lock", PackageManager::Deno);
    locks.insert("pnpm-lock.yaml", PackageManager::Pnpm);
    locks.insert("yarn.lock", PackageManager::Yarn);
    locks.insert("package-lock.json", PackageManager::Npm);
    locks.insert("npm-shrinkwrap.json", PackageManager::Npm);
    locks
}

pub fn get_install_page() -> HashMap<PackageManager, &'static str> {
    let mut install_page = HashMap::new();
    install_page.insert(PackageManager::Bun, "https://bun.sh");
    install_page.insert(PackageManager::Deno, "https://deno.com");
    install_page.insert(PackageManager::Pnpm, "https://pnpm.io/installation");
    install_page.insert(PackageManager::Pnpm6, "https://pnpm.io/6.x/installation");
    install_page.insert(PackageManager::Yarn, "https://classic.yarnpkg.com/en/docs/install");
    install_page.insert(PackageManager::YarnBerry, "https://yarnpkg.com/getting-started/install");
    install_page.insert(PackageManager::Npm, "https://docs.npmjs.com/cli/v8/configuring-npm/install");
    install_page
}


pub fn detect(cwd: &Path) -> Option<PackageManager> {
    let locks = get_locks();
    for (lock, package_manager) in locks.iter() {
        if cwd.join(lock).exists() {
            return Some(*package_manager);
        }
    }

    // Fallback to checking for npm if no lockfile is found
    if env::var("PATH").ok().map_or(false, |path| path.contains("npm")) {
        return Some(PackageManager::Npm);
    }

    None
}

pub fn detect_sync(cwd: &Path) -> Option<PackageManager> {
    let locks = get_locks();
    for (lock, package_manager) in locks.iter() {
        if cwd.join(lock).exists() {
            return Some(*package_manager);
        }
    }

    // Fallback to checking for npm if no lockfile is found
    if env::var("PATH").ok().map_or(false, |path| path.contains("npm")) {
        return Some(PackageManager::Npm);
    }

    None
}

pub fn get_user_agent() -> Option<PackageManager> {
    if let Ok(user_agent) = env::var("npm_config_user_agent") {
        let name = user_agent.split('/').next()?;
        match name {
            "npm" => Some(PackageManager::Npm),
            "yarn" => Some(PackageManager::Yarn),
            "pnpm" => Some(PackageManager::Pnpm),
            "bun" => Some(PackageManager::Bun),
            "deno" => Some(PackageManager::Deno),
            _ => None,
        }
    } else {
        None
    }
}

fn lookup(cwd: &str) -> Vec<PathBuf> {
    let mut directories = Vec::new();
    let mut directory = PathBuf::from(cwd);
    let root = Path::new("/");

    while directory.as_path() != root {
        directories.push(directory.clone());
        if !directory.pop() {
            break;
        }
    }
    directories
}

fn parse_package_json(filepath: &str) -> Option<PackageManager> {
    if !Path::new(filepath).exists() {
        return None;
    }

    handle_package_manager(filepath)
}

fn handle_package_manager(filepath: &str) -> Option<PackageManager> {
    if let Ok(contents) = fs::read_to_string(filepath) {
        if let Ok(json) = json::parse(&contents) {
            if let Some(package_manager) = json["packageManager"].as_str() {
                let parts: Vec<&str> = package_manager.split('@').collect();
                let name = parts[0];
                let version = parts.get(1).map(|&s| s).unwrap_or("latest");

                match name {
                    "yarn" if version.parse::<i32>().unwrap_or(0) > 1 => Some(PackageManager::YarnBerry),
                    "pnpm" if version.parse::<i32>().unwrap_or(0) < 7 => Some(PackageManager::Pnpm6),
                    "npm" => Some(PackageManager::Npm),
                    "yarn" => Some(PackageManager::Yarn),
                    "pnpm" => Some(PackageManager::Pnpm),
                    "bun" => Some(PackageManager::Bun),
                    "deno" => Some(PackageManager::Deno),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn file_exists(file_path: &str) -> bool {
    fs::metadata(file_path).is_ok()
}
