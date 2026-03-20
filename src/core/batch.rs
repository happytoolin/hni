use std::{
    path::Path,
    process::{Command, ExitCode, Stdio},
    thread,
};

use anyhow::{Context, Result};

use super::{shell::shell_escape, types::ResolvedExecution};

pub const INTERNAL_BATCH_PARALLEL: &str = "__hni_internal_batch_parallel";
pub const INTERNAL_BATCH_SEQUENTIAL: &str = "__hni_internal_batch_sequential";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    Parallel,
    Sequential,
}

impl BatchMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Parallel => "parallel",
            Self::Sequential => "sequential",
        }
    }

    pub fn internal_program(self) -> &'static str {
        match self {
            Self::Parallel => INTERNAL_BATCH_PARALLEL,
            Self::Sequential => INTERNAL_BATCH_SEQUENTIAL,
        }
    }

    pub fn from_internal_program(program: &str) -> Option<Self> {
        match program {
            INTERNAL_BATCH_PARALLEL => Some(Self::Parallel),
            INTERNAL_BATCH_SEQUENTIAL => Some(Self::Sequential),
            _ => None,
        }
    }
}

pub fn make_execution(mode: BatchMode, commands: Vec<String>, cwd: &Path) -> ResolvedExecution {
    ResolvedExecution::external(
        mode.internal_program().to_string(),
        commands,
        cwd.to_path_buf(),
        false,
    )
}

pub fn run_batch(mode: BatchMode, commands: &[String], cwd: &Path) -> Result<ExitCode> {
    if commands.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }

    match mode {
        BatchMode::Sequential => run_sequential(commands, cwd),
        BatchMode::Parallel => run_parallel(commands, cwd),
    }
}

pub fn format_batch_debug(mode: BatchMode, commands: &[String]) -> String {
    let mut rendered = Vec::with_capacity(commands.len() + 2);
    rendered.push("hni".to_string());
    rendered.push(format!("batch:{}", mode.label()));
    rendered.extend(commands.iter().cloned());
    rendered
        .iter()
        .map(|item| shell_escape(item))
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_sequential(commands: &[String], cwd: &Path) -> Result<ExitCode> {
    for command_string in commands {
        let status = shell_command(command_string, cwd)
            .status()
            .with_context(|| format!("failed to run command: {command_string}"))?;

        if !status.success() {
            return Ok(exit_code_from_status(status.code()));
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn run_parallel(commands: &[String], cwd: &Path) -> Result<ExitCode> {
    let mut handles = Vec::with_capacity(commands.len());
    for command_string in commands {
        let command_string = command_string.clone();
        let cwd = cwd.to_path_buf();
        handles.push(thread::spawn(move || -> Result<i32> {
            let status = shell_command(&command_string, &cwd)
                .status()
                .with_context(|| format!("failed to run command: {command_string}"))?;
            Ok(status.code().unwrap_or(1))
        }));
    }

    let mut first_non_zero: Option<i32> = None;
    for handle in handles {
        let code = handle
            .join()
            .map_err(|_| anyhow::anyhow!("parallel command worker panicked"))??;
        if code != 0 && first_non_zero.is_none() {
            first_non_zero = Some(code);
        }
    }

    Ok(first_non_zero.map_or(ExitCode::SUCCESS, exit_code_from_code))
}

fn shell_command(command_string: &str, cwd: &Path) -> Command {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command_string]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command_string]);
        cmd
    };

    cmd.current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    cmd
}

fn exit_code_from_status(code: Option<i32>) -> ExitCode {
    code.map_or_else(|| ExitCode::from(1), exit_code_from_code)
}

fn exit_code_from_code(code: i32) -> ExitCode {
    let code = u8::try_from(code).unwrap_or(1);
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn formats_parallel_debug() {
        let rendered = format_batch_debug(
            BatchMode::Parallel,
            &["echo hello world".to_string(), "echo ok".to_string()],
        );
        assert!(rendered.starts_with("hni batch:parallel"));
        assert!(
            rendered.contains("\"echo hello world\"") || rendered.contains("'echo hello world'")
        );
    }

    #[test]
    fn formats_sequential_debug() {
        let rendered = format_batch_debug(BatchMode::Sequential, &["echo one".to_string()]);
        assert!(rendered.starts_with("hni batch:sequential"));
    }

    #[test]
    fn batch_mode_roundtrip_with_internal_program() {
        for mode in [BatchMode::Parallel, BatchMode::Sequential] {
            let parsed = BatchMode::from_internal_program(mode.internal_program());
            assert_eq!(parsed, Some(mode));
        }
        assert_eq!(BatchMode::from_internal_program("not-batch"), None);
    }

    #[test]
    fn make_execution_sets_expected_internal_program() {
        let cwd = PathBuf::from("/tmp");
        let exec = make_execution(BatchMode::Parallel, vec!["echo hi".to_string()], &cwd);
        assert_eq!(exec.program, INTERNAL_BATCH_PARALLEL);
        assert_eq!(exec.args, vec!["echo hi"]);
        assert_eq!(exec.cwd, cwd);
        assert!(!exec.passthrough);
    }

    #[test]
    fn run_batch_with_no_commands_succeeds() {
        let cwd = PathBuf::from("/tmp");
        let code_parallel = run_batch(BatchMode::Parallel, &[], &cwd).unwrap();
        let code_sequential = run_batch(BatchMode::Sequential, &[], &cwd).unwrap();
        assert_eq!(code_parallel, ExitCode::SUCCESS);
        assert_eq!(code_sequential, ExitCode::SUCCESS);
    }
}
