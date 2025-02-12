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

trait CommandExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>>;
    fn install(&self, args: Vec<&str>) -> Option<Vec<String>>;
    fn add(&self, args: Vec<&str>) -> Option<Vec<String>>;
    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>>;
    // Add other command methods here
}

trait PackageManagerFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor>;
}

struct NpmExecutor {}

impl CommandExecutor for NpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        if args.len() > 1 {
            Some(vec!["npm".to_string(), "run".to_string(), args[0].to_string(), "--".to_string(), args.clone().into_iter().skip(1).collect::<Vec<&str>>().join(" ").to_string()])
        } else {
            Some(vec!["npm".to_string(), "run".to_string(), args[0].to_string()])
        }
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npm".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npm".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["npx".to_string(), args.join(" ").to_string()])
    }
}

struct NpmFactory {}

impl PackageManagerFactory for NpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(NpmExecutor {})
    }
}

// Yarn
struct YarnExecutor {}

impl CommandExecutor for YarnExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "run".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["yarn".to_string(), "exec".to_string(), args.join(" ").to_string()])
    }
}

struct YarnFactory {}

impl PackageManagerFactory for YarnFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(YarnExecutor {})
    }
}

// Pnpm
struct PnpmExecutor {}

impl CommandExecutor for PnpmExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "run".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["pnpm".to_string(), "dlx".to_string(), args.join(" ").to_string()])
    }
}

struct PnpmFactory {}

impl PackageManagerFactory for PnpmFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(PnpmExecutor {})
    }
}

// Bun
struct BunExecutor {}

impl CommandExecutor for BunExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "run".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["bun".to_string(), "x".to_string(), args.join(" ").to_string()])
    }
}

struct BunFactory {}

impl PackageManagerFactory for BunFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(BunExecutor {})
    }
}

// Deno
fn deno_execute(args: Vec<&str>) -> Option<Vec<String>> {
    Some(vec!["deno".to_string(), "run".to_string(), format!("npm:{}", args[0])])
}

struct DenoExecutor {}

impl CommandExecutor for DenoExecutor {
    fn run(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["deno".to_string(), "task".to_string(), args.join(" ").to_string()])
    }

    fn install(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["deno".to_string(), "install".to_string(), args.join(" ").to_string()])
    }

    fn add(&self, args: Vec<&str>) -> Option<Vec<String>> {
        Some(vec!["deno".to_string(), "add".to_string(), args.join(" ").to_string()])
    }

    fn execute(&self, args: Vec<&str>) -> Option<Vec<String>> {
        deno_execute(args)
    }
}

struct DenoFactory {}

impl PackageManagerFactory for DenoFactory {
    fn create_commands(&self) -> Box<dyn CommandExecutor> {
        Box::new(DenoExecutor {})
    }
}

pub fn resolve_command(agent: PackageManager, command: &str, args: Vec<&str>) -> Option<Vec<String>> {
    let factory: Box<dyn PackageManagerFactory> = match agent {
        PackageManager::Npm => Box::new(NpmFactory {}),
        PackageManager::Yarn => Box::new(YarnFactory {}),
        PackageManager::Pnpm => Box::new(PnpmFactory {}),
        PackageManager::Bun => Box::new(BunFactory {}),
        PackageManager::Deno => Box::new(DenoFactory {}),
        PackageManager::YarnBerry => Box::new(YarnFactory {}),
        PackageManager::Pnpm6 => Box::new(PnpmFactory {}),
        _ => return None,
    };

    let commands = factory.create_commands();

    match command {
        "run" => commands.run(args),
        "install" => commands.install(args),
        "add" => commands.add(args),
        "execute" => {
            if agent == PackageManager::Deno {
                deno_execute(args)
            } else {
                commands.execute(args)
            }
        }
        // Add other commands here
        _ => None,
    }
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
