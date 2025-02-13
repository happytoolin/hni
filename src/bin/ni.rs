use std::env;
use anyhow::Result;
use nirs::execute_command;
use log::{info, LevelFilter};
use env_logger::Builder;

fn main() -> Result<()> {
    // Initialize logger
    Builder::new()
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .init();

    let cwd = env::current_dir()?;
    let args: Vec<String> = env::args().skip(1).collect();
    
    // Split each argument individually to preserve spaces between arguments
    let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

    info!("Running ni command with arguments: {:?}", args);
    let mut cmd = execute_command(&cwd, "install", args)?;
    
    info!("Executing: {} {:?}", 
        cmd.get_program().to_string_lossy(),
        cmd.get_args().collect::<Vec<_>>()
    );
    let status = cmd.status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
