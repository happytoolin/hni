use anyhow::{Context, Result};
use log::{debug, info, warn};
use nirs::{detect::detect, execute_command, logger, NirsCommand};
use std::env;

struct NaCommand;

impl NirsCommand for NaCommand {
    fn run(&self, args: Vec<String>) -> Result<()> {
        let cwd = env::current_dir()?;
        debug!("Current working directory: {}", cwd.display());

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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with nice formatting
    logger::init();

    // Check for updates
    if let Err(e) = nirs::update_checker::check_for_update().await {
        warn!("Update check failed: {}", e);
    }

    let args: Vec<String> = env::args().skip(1).collect();
    let command = NaCommand;
    command.run(args)
}
