use clap::{Parser, Subcommand};
use duct::cmd;
use std::path::Path;

// Import the traits and factories
use nirs::package_managers::bun::BunFactory;
use nirs::package_managers::deno::DenoFactory;
use nirs::package_managers::npm::NpmFactory;
use nirs::package_managers::pnpm::PnpmFactory;
use nirs::package_managers::pnpm6::Pnpm6Factory;
use nirs::package_managers::yarn::YarnFactory;
use nirs::package_managers::yarn_berry::YarnBerryFactory;
use nirs::{PackageManagerFactory, ResolvedCommand};

#[derive(Parser)]
#[command(name = "ni", version, about = "Rust port of the ni tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install dependencies interactively
    Ni {
        /// Packages to install
        packages: Vec<String>,
        /// Install as development dependency
        #[arg(short, long)]
        dev: bool,
    },
    /// Run a script with history support
    Nr {
        /// The script to execute
        script: String,
    },
    // Additional commands: nci, na, nu, nun, nlx, etc.
}

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

    pub fn detect(cwd: &Path) -> Option<Self> {
        if cwd.join("pnpm-lock.yaml").exists() && !cwd.join(".pnpmfile.cjs").exists() {
            return Some(Self::Pnpm);
        }
        if cwd.join("yarn.lock").exists() && !cwd.join(".yarnrc.yml").exists() {
            return Some(Self::Yarn);
        }
        if cwd.join("yarn.lock").exists() && cwd.join(".yarnrc.yml").exists() {
            return Some(Self::YarnBerry);
        }
        if cwd.join(".pnpmfile.cjs").exists() && !cwd.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        }
        if cwd.join("package-lock.json").exists() {
            return Some(Self::Npm);
        }
        if cwd.join("npm-shrinkwrap.json").exists() {
            return Some(Self::Npm);
        }
        if cwd.join("bun.lockb").exists() {
            return Some(Self::Bun);
        }
        if cwd.join("bun.lock").exists() {
            return Some(Self::Bun);
        }
        if cwd.join("deno.lock").exists() {
            return Some(Self::Deno);
        }
        if cwd.join("pnpm-lock.yaml").exists() && cwd.join(".pnpmfile.cjs").exists() {
            return Some(Self::Pnpm6);
        }
        None
    }
}

pub fn parse_ni(agent: PackageManager, args: &[String]) -> ResolvedCommand {
    let factory: Box<dyn PackageManagerFactory> = match agent {
        PackageManager::Npm => Box::new(NpmFactory {}),
        PackageManager::Yarn => Box::new(YarnFactory {}),
        PackageManager::Pnpm => Box::new(PnpmFactory {}),
        PackageManager::Bun => Box::new(BunFactory {}),
        PackageManager::Deno => Box::new(DenoFactory {}),
        PackageManager::YarnBerry => Box::new(YarnBerryFactory {}),
        PackageManager::Pnpm6 => Box::new(Pnpm6Factory {}),
    };

    let commands = factory.create_commands();
    let command_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    commands
        .add(command_args)
        .unwrap_or_else(|| ResolvedCommand {
            bin: agent.as_str().to_string(),
            args: args.to_vec(),
        })
}

pub fn execute(command: ResolvedCommand, cwd: &Path) -> anyhow::Result<()> {
    cmd(command.bin, command.args).dir(cwd).run()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Ni { packages, dev: _ } => {
            use std::env;
            let cwd = env::current_dir()?;
            let manager = PackageManager::detect(&cwd).unwrap_or(PackageManager::Npm);
            let args: Vec<String> = packages;
            let resolved = parse_ni(manager, &args);
            execute(resolved, &cwd)?;
        }
        Commands::Nr { script: _ } => {
            // Dispatch to run command implementation
        } // Additional command dispatching...
    }
    Ok(())
}
