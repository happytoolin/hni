use anyhow::Result;
use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::ResolvedCommand;

pub async fn run(command: ResolvedCommand) -> Result<()> {
    let cwd = std::env::current_dir()?;
    log::info!("Running command: {} {:?}", command.bin, command.args);

    let mut cmd = Command::new(command.bin);
    cmd.args(&command.args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(cwd);

    let status = cmd.status()?;

    if status.success() {
        log::info!("Command executed successfully");
    } else {
        log::error!("Command failed");
    }

    Ok(())
}

pub fn execute(command: ResolvedCommand, cwd: &Path) -> Result<()> {
    Ok(())
}
