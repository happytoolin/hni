use duct::cmd;
use crate::parse::ResolvedCommand;
use std::path::Path;

pub fn execute(command: ResolvedCommand, cwd: &Path) -> anyhow::Result<()> {
    cmd(command.bin, command.args)
        .dir(cwd)
        .run()?;
    Ok(())
}
