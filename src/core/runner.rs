use std::process::{Command, ExitCode, Stdio};

use anyhow::{Context, Result};

use super::{
    batch::{format_batch_debug, run_batch, BatchMode},
    shell::shell_escape,
    types::ResolvedExecution,
};
use crate::platform::node::{resolve_real_node_path, SHIM_ACTIVE_ENV};

pub fn run(exec: &ResolvedExecution, use_sfw: bool) -> Result<ExitCode> {
    if let Some(mode) = BatchMode::from_internal_program(&exec.program) {
        return run_batch(mode, &exec.args, &exec.cwd);
    }

    let (program, args, passthrough) = materialize(exec, use_sfw)?;

    let mut command = Command::new(&program);
    command
        .args(&args)
        .current_dir(&exec.cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if passthrough {
        command.env(SHIM_ACTIVE_ENV, "1");
    }

    let status = command
        .status()
        .with_context(|| format!("failed to execute {program}"))?;

    let code = status.code().unwrap_or(1) as u8;
    Ok(ExitCode::from(code))
}

pub fn format_debug(exec: &ResolvedExecution, use_sfw: bool) -> Result<String> {
    if let Some(mode) = BatchMode::from_internal_program(&exec.program) {
        return Ok(format_batch_debug(mode, &exec.args));
    }

    let (program, args, _) = materialize(exec, use_sfw)?;
    let rendered = std::iter::once(shell_escape(&program))
        .chain(args.iter().map(|arg| shell_escape(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    Ok(rendered)
}

fn materialize(exec: &ResolvedExecution, use_sfw: bool) -> Result<(String, Vec<String>, bool)> {
    let mut program = exec.program.clone();
    let mut args = exec.args.clone();
    let mut passthrough = exec.passthrough;

    if exec.passthrough && exec.program == "node" {
        let real = resolve_real_node_path()?;
        program = real.to_string_lossy().to_string();
        passthrough = true;
    }

    if use_sfw && !exec.passthrough {
        let mut sfw_args = Vec::with_capacity(args.len() + 1);
        sfw_args.push(program);
        sfw_args.append(&mut args);
        program = "sfw".to_string();
        args = sfw_args;
        passthrough = false;
    }

    Ok((program, args, passthrough))
}
