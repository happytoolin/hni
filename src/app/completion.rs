use std::io;

use anyhow::{anyhow, Result};
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};

use crate::core::types::InvocationKind;

use super::help::help_command;

pub fn print_completion(shell: Option<&str>, program: &str) -> Result<()> {
    let shell = shell
        .map(str::to_owned)
        .or_else(detect_shell_from_env)
        .ok_or_else(|| anyhow!("missing shell; use one of: bash, zsh, fish"))?;

    let mut cmd = help_command(InvocationKind::Hni);
    let mut out = io::stdout();

    match shell.as_str() {
        "bash" => generate(Bash, &mut cmd, program, &mut out),
        "zsh" => generate(Zsh, &mut cmd, program, &mut out),
        "fish" => generate(Fish, &mut cmd, program, &mut out),
        _ => return Err(anyhow!("unsupported shell '{shell}'; use: bash, zsh, fish")),
    }

    Ok(())
}

fn detect_shell_from_env() -> Option<String> {
    let shell = std::env::var("SHELL").ok()?;
    let name = std::path::Path::new(&shell)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)?;
    Some(name.to_ascii_lowercase())
}
