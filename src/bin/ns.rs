use anyhow::Result;
use log::{debug, error, info, warn};
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    nirs::logger::init();

    let cwd = env::current_dir()?;
    debug!("Current working directory: {}", cwd.display());

    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        warn!("No commands provided to run sequentially.");
        return Ok(());
    }

    info!("Running commands sequentially...");

    let mut failed = false;

    for command_string in args {
        debug!("Executing command: {}", command_string);
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&command_string);
        cmd.current_dir(cwd.clone());

        let status = cmd.status()?;

        if status.success() {
            info!("Command completed successfully!");
        } else {
            error!("Command failed with exit code: {:?}", status.code());
            failed = true;
            break; // Stop on error
        }
    }

    if failed {
        std::process::exit(1);
    }

    info!("All commands completed!");
    Ok(())
}
