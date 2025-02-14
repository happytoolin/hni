use anyhow::{Context, Result};
use log::{debug, info, warn};
use nirs::{detect::detect, execute_command, logger};
use std::env;

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    logger::init();

    let cwd = env::current_dir()?;
    debug!("Current working directory: {}", cwd.display());

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        info!("No arguments provided, will show package manager info");
    }

    // Split each argument individually to preserve spaces between arguments
    let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    debug!("Parsed arguments: {:?}", args);

    // Detect package manager first
    let package_manager = detect(&cwd)?.context("Failed to detect package manager")?;
    info!("Using package manager: {}", package_manager);

    // For showing version info, use the run command with -v/--version
    let version_args = vec!["--version"];
    info!("Getting version information");
    let mut cmd = execute_command(&cwd, "run", version_args)?;

    info!(
        "Executing: {} with args: {:?}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args().collect::<Vec<_>>()
    );

    debug!("Starting command execution...");
    let status = cmd.status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        warn!("Command failed with exit code: {}", code);
        std::process::exit(code);
    }

    info!("Command completed successfully!");
    Ok(())
}
