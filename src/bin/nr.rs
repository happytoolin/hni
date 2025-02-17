use anyhow::Result;
use log::{debug, info, warn};
use nirs::{execute_command, NirsCommand};
use std::env;

struct NrCommand;

impl NirsCommand for NrCommand {
    fn run(&self, args: Vec<String>) -> Result<()> {
        let cwd = env::current_dir()?;
        debug!("Current working directory: {}", cwd.display());

        if args.is_empty() {
            warn!("No script name provided, will try to run 'start' script");
        }

        // Split each argument individually to preserve spaces between arguments
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        debug!("Parsed arguments: {:?}", args);

        info!("Running script with arguments: {:?}", args);
        let mut cmd = execute_command(&cwd, "run", args)?;

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
    // Initialize logger::init();

    let args: Vec<String> = env::args().skip(1).collect();
    let command = NrCommand;
    command.run(args)
}
