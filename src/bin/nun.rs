use anyhow::Result;
use log::{debug, info, warn};
use nirs::{execute_command, logger, NirsCommand};
use std::env;

struct NunCommand;

impl NirsCommand for NunCommand {
    fn run(&self, args: Vec<String>) -> Result<()> {
        let cwd = env::current_dir()?;
        debug!("Current working directory: {}", cwd.display());

        if args.is_empty() {
            warn!("No packages specified for uninstallation");
            std::process::exit(1);
        }

        // Split each argument individually to preserve spaces between arguments
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        debug!("Parsed arguments: {:?}", args);

        info!("Uninstalling packages: {:?}", args);
        let mut cmd = execute_command(&cwd, "uninstall", args)?;

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

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    logger::init();

    let args: Vec<String> = env::args().skip(1).collect();
    let command = NunCommand;
    command.run(args)
}
