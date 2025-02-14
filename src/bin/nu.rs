use anyhow::Result;
use log::{debug, info, warn};
use nirs::{execute_command, logger};
use std::env;

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    logger::init();

    let cwd = env::current_dir()?;
    debug!("Current working directory: {}", cwd.display());

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        info!("No packages specified, will upgrade all packages");
    }

    // Split each argument individually to preserve spaces between arguments
    let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    debug!("Parsed arguments: {:?}", args);

    info!("Upgrading packages: {:?}", args);
    let mut cmd = execute_command(&cwd, "upgrade", args)?;

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
