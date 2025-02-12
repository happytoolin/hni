use anyhow::Result;
use duct::cmd;
use std::path::Path;

use crate::ResolvedCommand;

pub async fn run(command: ResolvedCommand) -> Result<()> {
    let cwd = std::env::current_dir()?;
    println!("Running command: {} {:?}", command.bin, command.args);
    cmd(command.bin, command.args).dir(cwd).run()?;
    Ok(())
}

pub fn execute(command: ResolvedCommand, cwd: &Path) -> Result<()> {
    cmd(command.bin, command.args).dir(cwd).run()?;
    Ok(())
}
