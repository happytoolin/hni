use clap::{Parser, Subcommand};
use duct::cmd;
use std::path::Path;
use which::which;

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

pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    pub fn detect(cwd: &Path) -> Option<Self> {
        if cwd.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        }
        if cwd.join("yarn.lock").exists() {
            return Some(Self::Yarn);
        }
        if which("npm").is_ok() {
            return Some(Self::Npm);
        }
        None
    }
}

pub struct ResolvedCommand {
    pub bin: String,
    pub args: Vec<String>,
}

pub fn parse_ni(agent: PackageManager, args: &[String]) -> ResolvedCommand {
    match agent {
        PackageManager::Npm => {
            let args_str = args.join(" ");
            ResolvedCommand {
                bin: "npm".into(),
                args: vec!["install".into(), args_str],
            }
        }
        PackageManager::Yarn => {
            let args_str = args.join(" ");
            ResolvedCommand {
                bin: "yarn".into(),
                args: vec!["add".into(), args_str],
            }
        }
        PackageManager::Pnpm => {
            let args_str = args.join(" ");
            ResolvedCommand {
                bin: "pnpm".into(),
                args: vec!["add".into(), args_str],
            }
        }
        PackageManager::Bun => {
            let args_str = args.join(" ");
            ResolvedCommand {
                bin: "bun".into(),
                args: vec!["add".into(), args_str],
            }
        }
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
        }
        // Additional command dispatching...
    }
    Ok(())
}
