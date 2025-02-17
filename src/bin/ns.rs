use anyhow::Result;
use log::{debug, error, info, warn};
use nirs::{logger, NirsCommand};
use std::env;
use std::process::Command;

struct NsCommand;

impl NirsCommand for NsCommand {
    fn run(&self, args: Vec<String>) -> Result<()> {
        let cwd = env::current_dir()?;
        debug!("Current working directory: {}", cwd.display());

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
}

fn main() -> Result<()> {
    // Initialize logger with nice formatting
    logger::init();

    let args: Vec<String> = env::args().skip(1).collect();
    let command = NsCommand;
    command.run(args)
}
