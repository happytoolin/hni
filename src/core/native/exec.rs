use std::{
    path::Path,
    process::{Command, ExitCode},
};

use anyhow::{Context, Result};

use crate::{
    core::{
        shell::{configure_command, shell_command, shell_escape},
        types::{
            ExecutionStrategy, NativeExecution, NativeLocalBinExecution, NativeLocalBinLauncher,
            NativeScriptExecution, ResolvedExecution,
        },
    },
    platform::node::resolve_real_node_path,
};

use super::env::{apply_native_environment, native_script_env};

pub(super) fn run_script(exec: &NativeScriptExecution, invocation_cwd: &Path) -> Result<ExitCode> {
    if exec.steps.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }

    let shared_env = native_script_env(exec, invocation_cwd)?;

    for step in &exec.steps {
        let forwarded_args = if step.forward_args {
            exec.forwarded_args.as_slice()
        } else {
            &[]
        };

        let status =
            prepare_script_step_command(&step.command, forwarded_args, &exec.package_root)?
                .envs(shared_env.iter().map(|(key, value)| (key, value)))
                .env("npm_lifecycle_event", &step.event_name)
                .env("npm_lifecycle_script", &step.command)
                .status()
                .with_context(|| {
                    format!("failed to execute native script step '{}'", step.event_name)
                })?;

        if !status.success() {
            return Ok(exit_code_from_status(status.code()));
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub(super) fn run_local_bin(exec: &NativeLocalBinExecution, cwd: &Path) -> Result<ExitCode> {
    let mut command = spawn_local_bin_command(exec)?;
    command.current_dir(cwd);

    apply_native_environment(&mut command, &exec.bin_paths)?;

    let status = command
        .status()
        .with_context(|| format!("failed to execute native local bin {}", exec.bin_name))?;

    Ok(exit_code_from_status(status.code()))
}

pub(super) fn format_debug(exec: &ResolvedExecution) -> String {
    match &exec.strategy {
        ExecutionStrategy::Native(NativeExecution::RunScript(native)) => {
            let mut rendered = vec![
                "hni".to_string(),
                "native:run-script".to_string(),
                native.script_name.clone(),
            ];
            rendered.extend(native.forwarded_args.clone());
            join_rendered(rendered)
        }
        ExecutionStrategy::Native(NativeExecution::RunLocalBin(native)) => {
            let mut rendered = vec![
                "hni".to_string(),
                "native:run-local-bin".to_string(),
                native.bin_name.clone(),
            ];
            rendered.extend(native.forwarded_args.clone());
            join_rendered(rendered)
        }
        ExecutionStrategy::External => String::new(),
    }
}

fn join_rendered(rendered: Vec<String>) -> String {
    rendered
        .iter()
        .map(|part| shell_escape(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn spawn_local_bin_command(exec: &NativeLocalBinExecution) -> Result<Command> {
    let mut command = match &exec.launcher {
        NativeLocalBinLauncher::Binary(path) => Command::new(path),
        NativeLocalBinLauncher::Cmd(path) => {
            let mut command = Command::new("cmd");
            command.arg("/C").arg(path);
            command
        }
        NativeLocalBinLauncher::PowerShell(path) => {
            let mut command = Command::new("powershell");
            command
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(path);
            command
        }
        NativeLocalBinLauncher::NodeScript {
            script_path,
            node_args,
        } => {
            let mut command = Command::new(resolve_real_node_path()?);
            command.args(node_args).arg(script_path);
            command
        }
    };

    command.args(&exec.forwarded_args);
    Ok(configure_command(command, Path::new(".")))
}

fn prepare_script_step_command(
    command_string: &str,
    forwarded_args: &[String],
    cwd: &Path,
) -> Result<Command> {
    if let Some(command) = spawn_direct_script_command(command_string, forwarded_args)? {
        return Ok(configure_command(command, cwd));
    }

    Ok(spawn_shell_command(command_string, forwarded_args, cwd))
}

fn spawn_direct_script_command(
    command_string: &str,
    forwarded_args: &[String],
) -> Result<Option<Command>> {
    if cfg!(windows) || contains_shell_metacharacters(command_string) {
        return Ok(None);
    }

    let Some(tokens) = shlex::split(command_string) else {
        return Ok(None);
    };
    let Some((program, args)) = tokens.split_first() else {
        return Ok(None);
    };

    if looks_like_env_assignment(program) {
        return Ok(None);
    }

    let mut command = if is_node_program(program) {
        Command::new(resolve_real_node_path()?)
    } else {
        Command::new(program)
    };
    command.args(args).args(forwarded_args);
    Ok(Some(command))
}

fn spawn_shell_command(command_string: &str, forwarded_args: &[String], cwd: &Path) -> Command {
    let line = build_shell_command_line(command_string, forwarded_args);
    configure_command(shell_command(&line), cwd)
}

fn build_shell_command_line(command_string: &str, forwarded_args: &[String]) -> String {
    if forwarded_args.is_empty() {
        return command_string.to_string();
    }

    let escaped_args = forwarded_args
        .iter()
        .map(|arg| shell_arg_escape(arg))
        .collect::<Vec<_>>()
        .join(" ");

    format!("{command_string} {escaped_args}")
}

#[cfg(not(windows))]
fn shell_arg_escape(arg: &str) -> String {
    shell_escape(arg)
}

#[cfg(windows)]
fn shell_arg_escape(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }

    if arg.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '=' | '@' | '\\')
    }) {
        return arg.to_string();
    }

    format!("\"{}\"", arg.replace('"', "\"\""))
}

fn contains_shell_metacharacters(command_string: &str) -> bool {
    command_string.chars().any(|ch| {
        matches!(
            ch,
            '|' | '&'
                | ';'
                | '<'
                | '>'
                | '('
                | ')'
                | '$'
                | '`'
                | '*'
                | '?'
                | '['
                | ']'
                | '{'
                | '}'
                | '~'
                | '\n'
                | '\r'
        )
    })
}

fn looks_like_env_assignment(token: &str) -> bool {
    token.contains('=') && !token.starts_with('-')
}

fn is_node_program(program: &str) -> bool {
    Path::new(program)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            value.eq_ignore_ascii_case("node") || value.eq_ignore_ascii_case("node.exe")
        })
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
    use std::fs;

    use super::*;
    use crate::core::types::NativeScriptStep;

    #[test]
    fn glob_patterns_force_shell_fallback() {
        assert!(contains_shell_metacharacters("printf \"%s\\n\" src/*.js"));
        assert!(
            spawn_direct_script_command("printf \"%s\\n\" src/*.js", &[])
                .unwrap()
                .is_none()
        );
    }

    #[cfg(windows)]
    #[test]
    fn direct_script_fast_path_is_disabled_on_windows() {
        assert!(
            spawn_direct_script_command(r"node .\\scripts\\build.js", &[])
                .unwrap()
                .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn shell_fallback_does_not_leak_forwarded_args_into_shell_positionals() {
        let dir = tempfile::tempdir().unwrap();
        let package_json_path = dir.path().join("package.json");
        fs::write(&package_json_path, "{}").unwrap();
        let output_path = dir.path().join("count.txt");
        let capture_script = dir.path().join("capture.sh");
        fs::write(
            &capture_script,
            format!(
                "#!/bin/sh\nprintf '%s' \"$#\" > {}\n",
                shell_escape(output_path.to_str().unwrap())
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = fs::metadata(&capture_script).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&capture_script, perms).unwrap();
        }
        let command = format!(
            "{} \"$@\" && :",
            shell_escape(capture_script.to_str().unwrap())
        );

        let exec = NativeScriptExecution {
            package_root: dir.path().to_path_buf(),
            package_json_path,
            script_name: "count".to_string(),
            steps: vec![NativeScriptStep {
                event_name: "count".to_string(),
                command,
                forward_args: true,
            }],
            forwarded_args: vec!["alpha".to_string(), "two words".to_string()],
            bin_paths: Vec::new(),
        };

        run_script(&exec, dir.path()).unwrap();
        assert_eq!(fs::read_to_string(output_path).unwrap(), "0");
    }
}
