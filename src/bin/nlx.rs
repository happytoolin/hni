use std::{env, process::Command};

use anyhow::Result;
use log::{debug, info, warn};
use nirs::{detect, parse_nlx};

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    nirs::logger::init();

    let cwd = env::current_dir()?;
    debug!("Current working directory: {}", cwd.display());

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        warn!("No command provided to execute");
        std::process::exit(1);
    }

    debug!("Parsed arguments: {:?}", args);
    info!("Executing command with arguments: {:?}", args);

    let agent = detect::detect(&cwd)?;
    let resolved = parse_nlx(agent.expect("No package manager detected"), &args)?;

    info!("Executing: {} with args: {:?}", resolved.bin, resolved.args);

    debug!("Starting command execution...");
    let status = Command::new(&resolved.bin).args(&resolved.args).status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        warn!("Command failed with exit code: {}", code);
        std::process::exit(code);
    }

    info!("Command completed successfully!");
    Ok(())
}
