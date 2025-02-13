use std::env;
use std::path::Path;

use anyhow::Result;

use nirs::execute_command;

fn main() -> Result<()> {
    let cwd = env::current_dir()?;
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut cmd = execute_command(&cwd, "run", args)?;
    let status = cmd.status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
