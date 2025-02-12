use clap::{Parser, Subcommand};
use duct::cmd;
use std::path::Path;
use which::which;
use std::path::PathBuf;
use std::fs;

// Import the traits and factories
use crate::npm::{CommandExecutor as NpmCommandExecutor, PackageManagerFactory as NpmPackageManagerFactory, NpmFactory};
use crate::yarn::{CommandExecutor as YarnCommandExecutor, PackageManagerFactory as YarnPackageManagerFactory, YarnFactory};
use crate::pnpm::{CommandExecutor as PnpmCommandExecutor, PackageManagerFactory as PnpmPackageManagerFactory, PnpmFactory};
use crate::bun::{CommandExecutor as BunCommandExecutor, PackageManagerFactory as BunPackageManagerFactory, BunFactory};
use crate::deno::{CommandExecutor as DenoCommandExecutor, PackageManagerFactory as DenoPackageManagerFactory, DenoFactory};

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
        if cwd.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        }
        if cwd.join("yarn.lock").exists() {
            return Some(Self::Yarn);
        }
        if cwd.join("bun.lockb").exists() {
            return Some(Self::Bun);
        }
        if cwd.join("bun.lock").exists() {
            return Some(Self::Bun);
        }
        if cwd.join("package-lock.json").exists() {
            return Some(Self::Npm);
        }
        if cwd.join("npm-shrinkwrap.json").exists() {
            return Some(Self::Npm);
        }
        if cwd.join("deno.lock").exists() {
            return Some(Self::Deno);
        }
        None
    }
}

pub struct ResolvedCommand {
    pub bin: String,
    pub args: Vec<String>,
}

pub fn parse_ni(agent: PackageManager, args: &[String]) -> ResolvedCommand {
    let factory: Box<dyn NpmPackageManagerFactory> = match agent {
        PackageManager::Npm => Box::new(NpmFactory {}),
        PackageManager::Yarn => Box::new(YarnFactory {}),
        PackageManager::Pnpm => Box::new(PnpmFactory {}),
        PackageManager::Bun => Box::new(BunFactory {}),
        PackageManager::Deno => Box::new(DenoFactory {}),
        PackageManager::YarnBerry => Box::new(YarnFactory {}),
        PackageManager::Pnpm6 => Box::new(PnpmFactory {}),
    };

    let commands = factory.create_commands();
    let command_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let resolved_command = commands.add(command_args).unwrap();

    ResolvedCommand {
        bin: resolved_command[0].clone(),
        args: resolved_command.into_iter().skip(1).collect(),
    }
}

pub fn execute(command: ResolvedCommand, cwd: &Path) -> anyhow::Result<()> {
    cmd(command.bin, command.args).dir(cwd).run()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
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
