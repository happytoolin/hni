use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use anyhow::{Context, Result};
use deno_task_shell::{KillSignal, execute, parser::parse};
use tokio::{runtime::Builder, task::LocalSet};

use crate::{
    core::{
        deno::{find_nearest_deno_project, plan_native_deno_task},
        error::HniResult,
        package::resolve_local_bin,
        resolve::ResolveContext,
        shell::shell_escape,
        types::{
            NativeDenoTaskExecution, NativeDenoTaskStage, NativeDenoTaskStep,
            NativeLocalBinExecution, NativeScriptExecution, NativeScriptStep, PackageManager,
            ResolvedExecution,
        },
    },
    platform::node::{REAL_NODE_ENV, path_with_real_node_priority, resolve_real_node_path},
};

const UNSUPPORTED_SCRIPT_PATTERNS: &[&str] = &["npm_package_", "npm_config_"];

pub enum NativeAttempt {
    Eligible(Box<ResolvedExecution>),
    Ineligible(String),
}

pub fn attempt_nr(
    pm: Option<PackageManager>,
    args: &[String],
    ctx: &ResolveContext,
    has_if_present: bool,
) -> HniResult<NativeAttempt> {
    let state = ctx.project_state()?;

    if pm == Some(PackageManager::Deno) {
        return attempt_deno_nr(args, ctx, has_if_present);
    }

    let Some(pkg) = state.nearest_package() else {
        return Ok(NativeAttempt::Ineligible(
            "native script execution requires a nearest package.json".to_string(),
        ));
    };

    let scripts = pkg.manifest.scripts.unwrap_or_default();
    let bin_paths = state.bin_dirs().to_vec();

    if pm == Some(PackageManager::YarnBerry) && state.has_yarn_pnp_loader() {
        return Ok(NativeAttempt::Ineligible(
            "yarn berry Plug'n'Play does not expose node_modules/.bin; falling back to yarn execution"
                .to_string(),
        ));
    }

    let script_name = args.first().cloned().unwrap_or_else(|| "start".to_string());
    let forwarded_args = args.iter().skip(1).cloned().collect::<Vec<_>>();
    let Some(script) = scripts.get(&script_name) else {
        if has_if_present {
            return Ok(NativeAttempt::Eligible(Box::new(
                ResolvedExecution::native_script(
                    script_name.clone(),
                    ctx.cwd().to_path_buf(),
                    NativeScriptExecution {
                        package_root: pkg.root,
                        package_json_path: pkg.package_json_path,
                        script_name,
                        steps: Vec::new(),
                        forwarded_args,
                        bin_paths,
                    },
                ),
            )));
        }

        return Ok(NativeAttempt::Ineligible(format!(
            "script '{script_name}' was not found in the nearest package.json"
        )));
    };

    let mut steps = Vec::new();

    if let Some(pre) = scripts.get(&format!("pre{script_name}")) {
        if let Some(reason) = unsupported_script_reason(&format!("pre{script_name}"), pre) {
            return Ok(NativeAttempt::Ineligible(reason));
        }
        steps.push(NativeScriptStep {
            event_name: format!("pre{script_name}"),
            command: pre.clone(),
            forward_args: false,
        });
    }

    if let Some(reason) = unsupported_script_reason(&script_name, script) {
        return Ok(NativeAttempt::Ineligible(reason));
    }
    steps.push(NativeScriptStep {
        event_name: script_name.clone(),
        command: script.clone(),
        forward_args: true,
    });

    if let Some(post) = scripts.get(&format!("post{script_name}")) {
        if let Some(reason) = unsupported_script_reason(&format!("post{script_name}"), post) {
            return Ok(NativeAttempt::Ineligible(reason));
        }
        steps.push(NativeScriptStep {
            event_name: format!("post{script_name}"),
            command: post.clone(),
            forward_args: false,
        });
    }

    let exec = NativeScriptExecution {
        package_root: pkg.root,
        package_json_path: pkg.package_json_path,
        script_name: script_name.clone(),
        steps,
        forwarded_args,
        bin_paths,
    };

    Ok(NativeAttempt::Eligible(Box::new(
        ResolvedExecution::native_script(script_name, ctx.cwd().to_path_buf(), exec),
    )))
}

pub fn attempt_nlx(
    pm: Option<PackageManager>,
    args: &[String],
    ctx: &ResolveContext,
) -> HniResult<NativeAttempt> {
    let state = ctx.project_state()?;

    let Some(bin_name) = args.first() else {
        return Ok(NativeAttempt::Ineligible(
            "native local bin execution requires a command".to_string(),
        ));
    };
    if pm == Some(PackageManager::YarnBerry) && state.has_yarn_pnp_loader() {
        return Ok(NativeAttempt::Ineligible(
            "yarn berry Plug'n'Play does not expose node_modules/.bin; falling back to yarn execution"
                .to_string(),
        ));
    }
    let bin_paths = state.bin_dirs().to_vec();
    let bin_path = resolve_local_bin(bin_name, &bin_paths)
        .or_else(|| state.resolve_declared_package_bin(bin_name));
    let Some(bin_path) = bin_path else {
        if pm == Some(PackageManager::Deno) {
            return Ok(NativeAttempt::Ineligible(
                "remote deno exec stays delegated".to_string(),
            ));
        }

        return Ok(NativeAttempt::Ineligible(
            "local binary not found in node_modules/.bin or package.json bin entries; falling back to package-manager exec"
                .to_string(),
        ));
    };

    let exec = NativeLocalBinExecution {
        bin_name: bin_name.clone(),
        bin_path,
        bin_paths,
    };

    Ok(NativeAttempt::Eligible(Box::new(
        ResolvedExecution::native_local_bin(
            bin_name.clone(),
            args.iter().skip(1).cloned().collect(),
            ctx.cwd().to_path_buf(),
            exec,
        ),
    )))
}

fn attempt_deno_nr(
    args: &[String],
    ctx: &ResolveContext,
    has_if_present: bool,
) -> HniResult<NativeAttempt> {
    let selection = args.first().cloned().unwrap_or_else(|| "start".to_string());
    let forwarded_args = args.iter().skip(1).cloned().collect::<Vec<_>>();
    let Some(project) = find_nearest_deno_project(ctx.cwd())? else {
        return Ok(NativeAttempt::Ineligible(
            "native deno execution requires a nearest deno project".to_string(),
        ));
    };

    match plan_native_deno_task(&project, &selection, &forwarded_args, has_if_present) {
        Ok(exec) => Ok(NativeAttempt::Eligible(Box::new(
            ResolvedExecution::native_deno_task(selection, ctx.cwd().to_path_buf(), exec),
        ))),
        Err(reason) => Ok(NativeAttempt::Ineligible(reason)),
    }
}

pub fn run_script(exec: &NativeScriptExecution, invocation_cwd: &Path) -> Result<ExitCode> {
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

pub fn run_deno_task(exec: &NativeDenoTaskExecution, invocation_cwd: &Path) -> Result<ExitCode> {
    let envs = deno_task_env(exec, invocation_cwd)?;

    for stage in &exec.stages {
        for step in &stage.steps {
            print_deno_task_line(step, &exec.forwarded_args);
        }

        let Some(command) = stage_command(stage, &exec.forwarded_args) else {
            continue;
        };

        let exit_code = execute_deno_shell_command(&command, &exec.project_root, &envs)?;
        if exit_code != 0 {
            return Ok(exit_code_from_code(exit_code));
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub fn run_local_bin(
    exec: &NativeLocalBinExecution,
    cwd: &Path,
    args: &[String],
) -> Result<ExitCode> {
    let mut command = spawn_local_bin_command(&exec.bin_path, args)?;
    command
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    apply_native_environment(&mut command, &exec.bin_paths)?;

    let status = command
        .status()
        .with_context(|| format!("failed to execute native local bin {}", exec.bin_name))?;

    Ok(exit_code_from_status(status.code()))
}

pub fn format_debug(exec: &ResolvedExecution) -> String {
    match &exec.strategy {
        crate::core::types::ExecutionStrategy::Native(
            crate::core::types::NativeExecution::RunScript(native),
        ) => {
            let mut rendered = vec![
                "hni".to_string(),
                "native:run-script".to_string(),
                native.script_name.clone(),
            ];
            if !native.forwarded_args.is_empty() {
                rendered.push("--".to_string());
                rendered.extend(native.forwarded_args.clone());
            }
            join_rendered(rendered)
        }
        crate::core::types::ExecutionStrategy::Native(
            crate::core::types::NativeExecution::RunDenoTask(native),
        ) => {
            let mut rendered = vec![
                "hni".to_string(),
                "native:run-deno-task".to_string(),
                native.selection.clone(),
            ];
            if !native.forwarded_args.is_empty() {
                rendered.push("--".to_string());
                rendered.extend(native.forwarded_args.clone());
            }
            join_rendered(rendered)
        }
        crate::core::types::ExecutionStrategy::Native(
            crate::core::types::NativeExecution::RunLocalBin(native),
        ) => {
            let mut rendered = vec![
                "hni".to_string(),
                "native:run-local-bin".to_string(),
                native.bin_name.clone(),
            ];
            rendered.extend(exec.args.clone());
            join_rendered(rendered)
        }
        crate::core::types::ExecutionStrategy::External => join_rendered(vec![]),
    }
}

fn deno_task_env(
    exec: &NativeDenoTaskExecution,
    invocation_cwd: &Path,
) -> Result<HashMap<OsString, OsString>> {
    let mut envs = std::env::vars_os().collect::<HashMap<_, _>>();
    envs.insert(
        OsString::from("INIT_CWD"),
        invocation_cwd.as_os_str().to_os_string(),
    );
    envs.insert(
        OsString::from("PATH"),
        OsString::from(merged_path_with_bins(&exec.bin_paths)?),
    );

    if let Ok(real_node) = resolve_real_node_path() {
        envs.insert(OsString::from(REAL_NODE_ENV), real_node.into_os_string());
    }

    Ok(envs)
}

fn stage_command(stage: &NativeDenoTaskStage, forwarded_args: &[String]) -> Option<String> {
    if stage.steps.is_empty() {
        return None;
    }

    let commands = stage
        .steps
        .iter()
        .map(|step| {
            let command = deno_task_command_string(
                &step.command,
                if step.forward_args {
                    forwarded_args
                } else {
                    &[]
                },
            );
            if stage.steps.len() == 1 {
                command
            } else {
                format!("({command})")
            }
        })
        .collect::<Vec<_>>();

    Some(if commands.len() == 1 {
        commands[0].clone()
    } else {
        commands.join(" & ")
    })
}

fn deno_task_command_string(command: &str, forwarded_args: &[String]) -> String {
    if forwarded_args.is_empty() {
        return command.to_string();
    }

    format!(
        "{command} -- {}",
        forwarded_args
            .iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn print_deno_task_line(step: &NativeDenoTaskStep, forwarded_args: &[String]) {
    if step.forward_args && !forwarded_args.is_empty() {
        println!(
            "Task {} {} -- {}",
            step.task_name,
            step.command,
            forwarded_args
                .iter()
                .map(|arg| shell_escape(arg))
                .collect::<Vec<_>>()
                .join(" ")
        );
    } else {
        println!("Task {} {}", step.task_name, step.command);
    }
}

fn execute_deno_shell_command(
    command: &str,
    cwd: &Path,
    envs: &HashMap<OsString, OsString>,
) -> Result<i32> {
    let parsed = parse(command).context("failed to parse deno task command")?;
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;
    let local_set = LocalSet::new();

    Ok(runtime.block_on(local_set.run_until(execute(
        parsed,
        envs.clone(),
        cwd.to_path_buf(),
        Default::default(),
        KillSignal::default(),
    ))))
}

fn join_rendered(rendered: Vec<String>) -> String {
    rendered
        .iter()
        .map(|part| shell_escape(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn unsupported_script_reason(event_name: &str, script: &str) -> Option<String> {
    for pattern in UNSUPPORTED_SCRIPT_PATTERNS {
        if script.contains(pattern) {
            return Some(format!(
                "script '{event_name}' uses unsupported native environment expansion ({pattern})"
            ));
        }
    }

    None
}

fn native_script_env(
    exec: &NativeScriptExecution,
    invocation_cwd: &Path,
) -> Result<Vec<(String, String)>> {
    let mut envs = Vec::with_capacity(4);
    envs.push((
        "INIT_CWD".to_string(),
        invocation_cwd.to_string_lossy().to_string(),
    ));
    envs.push((
        "npm_package_json".to_string(),
        exec.package_json_path.to_string_lossy().to_string(),
    ));

    if let Ok(current_exe) = env::current_exe() {
        envs.push((
            "npm_execpath".to_string(),
            current_exe.to_string_lossy().to_string(),
        ));
    }

    if let Ok(real_node) = resolve_real_node_path() {
        envs.push((
            "npm_node_execpath".to_string(),
            real_node.to_string_lossy().to_string(),
        ));
    }

    let merged_path = merged_path_with_bins(&exec.bin_paths)?;
    envs.push(("PATH".to_string(), merged_path));
    Ok(envs)
}

fn apply_native_environment(command: &mut Command, bin_paths: &[PathBuf]) -> Result<()> {
    if let Ok(path) = merged_path_with_bins(bin_paths) {
        command.env("PATH", path);
    }

    if let Ok(real_node) = resolve_real_node_path() {
        command.env(REAL_NODE_ENV, &real_node);
    }

    Ok(())
}

fn spawn_local_bin_command(bin_path: &Path, args: &[String]) -> Result<Command> {
    if let Some(node_launch) = resolve_node_bin_launcher(bin_path)? {
        let mut command = Command::new(resolve_real_node_path()?);
        command
            .args(node_launch.node_args)
            .arg(node_launch.script_path)
            .args(args);
        return Ok(command);
    }

    let extension = bin_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    match extension.as_deref() {
        Some("cmd") | Some("bat") => {
            let mut command = Command::new("cmd");
            command.arg("/C").arg(bin_path).args(args);
            Ok(command)
        }
        Some("ps1") => {
            let mut command = Command::new("powershell");
            command
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(bin_path)
                .args(args);
            Ok(command)
        }
        Some("js") | Some("cjs") | Some("mjs") => {
            let mut command = Command::new(resolve_real_node_path()?);
            command.arg(bin_path).args(args);
            Ok(command)
        }
        _ => {
            let mut command = Command::new(bin_path);
            command.args(args);
            Ok(command)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeBinLaunch {
    script_path: PathBuf,
    node_args: Vec<String>,
}

fn resolve_node_bin_launcher(bin_path: &Path) -> Result<Option<NodeBinLaunch>> {
    let inspected_path = resolve_bin_source_path(bin_path)?;

    if matches!(
        inspected_path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("js") | Some("cjs") | Some("mjs")
    ) {
        return Ok(Some(NodeBinLaunch {
            script_path: inspected_path,
            node_args: Vec::new(),
        }));
    }

    let raw = match fs::read_to_string(&inspected_path) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };

    if let Some(node_args) = raw.lines().next().and_then(node_args_from_shebang) {
        return Ok(Some(NodeBinLaunch {
            script_path: inspected_path,
            node_args,
        }));
    }

    Ok(parse_node_shell_shim(&raw, &inspected_path))
}

fn resolve_bin_source_path(bin_path: &Path) -> Result<PathBuf> {
    let mut current = bin_path.to_path_buf();

    for _ in 0..8 {
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(_) => return Ok(current),
        };

        if !metadata.file_type().is_symlink() {
            return Ok(current.canonicalize().unwrap_or(current));
        }

        let target = fs::read_link(&current)?;
        current = if target.is_absolute() {
            target
        } else {
            current
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(target)
        };
    }

    Ok(current.canonicalize().unwrap_or(current))
}

fn node_args_from_shebang(line: &str) -> Option<Vec<String>> {
    let shebang = line.strip_prefix("#!")?.trim();
    let mut tokens = shlex::split(shebang)?;
    if tokens.is_empty() {
        return None;
    }

    if is_env_program(&tokens[0]) {
        tokens.remove(0);
        if tokens.first().is_some_and(|token| token == "-S") {
            tokens.remove(0);
        }
        while tokens
            .first()
            .is_some_and(|token| looks_like_env_assignment(token))
        {
            tokens.remove(0);
        }
    }

    let program = tokens.first()?;
    if !is_node_program(program) {
        return None;
    }

    Some(tokens.into_iter().skip(1).collect())
}

fn parse_node_shell_shim(raw: &str, shim_path: &Path) -> Option<NodeBinLaunch> {
    let shim_dir = shim_path.parent()?;

    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("exec ") {
            continue;
        }

        let Some(tokens) = shlex::split(trimmed) else {
            continue;
        };

        if tokens.len() < 4 || tokens.first().map(String::as_str) != Some("exec") {
            continue;
        }

        let Some(program) = tokens.get(1) else {
            continue;
        };
        if !(is_node_program(program) || is_basedir_node_program(program)) {
            continue;
        }

        if tokens.last().map(String::as_str) != Some("$@") {
            continue;
        }

        let Some(script_token) = tokens.get(tokens.len() - 2) else {
            continue;
        };
        let Some(script_path) = resolve_shim_path_token(script_token, shim_dir) else {
            continue;
        };
        if !looks_like_node_script_path(&script_path) {
            continue;
        }

        return Some(NodeBinLaunch {
            script_path,
            node_args: tokens[2..tokens.len() - 2].to_vec(),
        });
    }

    None
}

fn resolve_shim_path_token(token: &str, shim_dir: &Path) -> Option<PathBuf> {
    if let Some(relative) = token.strip_prefix("$basedir/") {
        return Some(shim_dir.join(relative));
    }

    if let Some(relative) = token.strip_prefix("$basedir\\") {
        return Some(shim_dir.join(relative.replace('\\', "/")));
    }

    if !token.contains('/') && !token.contains('\\') {
        return None;
    }

    let path = Path::new(token);
    Some(if path.is_absolute() {
        path.to_path_buf()
    } else {
        shim_dir.join(path)
    })
}

fn looks_like_node_script_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("js") | Some("cjs") | Some("mjs")
    )
}

fn is_env_program(program: &str) -> bool {
    Path::new(program)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("env"))
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

fn is_basedir_node_program(program: &str) -> bool {
    matches!(program, "$basedir/node" | "$basedir/node.exe")
}

fn merged_path_with_bins(bin_paths: &[PathBuf]) -> Result<String> {
    let current_path = env::var_os("PATH");
    let mut ordered = bin_paths.to_vec();

    if let Ok(real_node) = resolve_real_node_path()
        && let Some(path) = path_with_real_node_priority(&real_node, current_path.clone())
    {
        ordered.extend(env::split_paths(&path));
        return join_paths_string(ordered);
    }

    if let Some(current_path) = current_path {
        ordered.extend(env::split_paths(&current_path));
    }

    join_paths_string(ordered)
}

fn join_paths_string(paths: Vec<PathBuf>) -> Result<String> {
    env::join_paths(paths)
        .map(|value| value.to_string_lossy().to_string())
        .map_err(Into::into)
}

fn prepare_script_step_command(
    command_string: &str,
    forwarded_args: &[String],
    cwd: &Path,
) -> Result<Command> {
    if let Some(command) = spawn_direct_script_command(command_string, forwarded_args)? {
        return Ok(configure_command(command, cwd));
    }

    Ok(shell_command(command_string, forwarded_args, cwd))
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

fn shell_command(command_string: &str, forwarded_args: &[String], cwd: &Path) -> Command {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.arg("/C").arg(command_string);
        command
    } else {
        let combined = if forwarded_args.is_empty() {
            command_string.to_string()
        } else {
            format!(
                "{command_string} {}",
                forwarded_args
                    .iter()
                    .map(|arg| shell_escape(arg))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let mut command = Command::new("sh");
        command.arg("-c").arg(combined).arg("sh");
        command
    };

    command.args(forwarded_args);
    configure_command(command, cwd)
}

fn configure_command(mut command: Command, cwd: &Path) -> Command {
    command
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
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
    use std::fs;

    #[test]
    fn parses_node_shebang_with_env_and_args() {
        let args =
            node_args_from_shebang("#!/usr/bin/env -S node --no-warnings --trace-deprecation")
                .unwrap();
        assert_eq!(args, vec!["--no-warnings", "--trace-deprecation"]);
    }

    #[test]
    fn ignores_non_node_shebangs() {
        assert_eq!(node_args_from_shebang("#!/usr/bin/env bash"), None);
    }

    #[test]
    fn parses_npm_style_shell_shim_exec_line() {
        let dir = tempfile::tempdir().unwrap();
        let shim = dir.path().join("node_modules").join(".bin").join("hello");
        fs::create_dir_all(shim.parent().unwrap()).unwrap();

        let raw = r#"#!/bin/sh
basedir=$(dirname "$(echo "$0" | sed -e 's,\\,/,g')")

if [ -x "$basedir/node" ]; then
  exec "$basedir/node" --no-warnings "$basedir/../hello/cli.js" "$@"
else 
  exec node --no-warnings "$basedir/../hello/cli.js" "$@"
fi
"#;

        let parsed = parse_node_shell_shim(raw, &shim).unwrap();
        assert_eq!(
            parsed,
            NodeBinLaunch {
                script_path: shim.parent().unwrap().join("../hello/cli.js"),
                node_args: vec!["--no-warnings".to_string()],
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_symlinked_js_bins_to_underlying_script() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let package_dir = dir.path().join("node_modules").join("tool");
        let bin_dir = dir.path().join("node_modules").join(".bin");
        fs::create_dir_all(&package_dir).unwrap();
        fs::create_dir_all(&bin_dir).unwrap();
        let script = package_dir.join("cli.js");
        fs::write(&script, "console.log('hi')").unwrap();
        let shim = bin_dir.join("tool");
        symlink("../tool/cli.js", &shim).unwrap();

        let parsed = resolve_node_bin_launcher(&shim).unwrap().unwrap();
        assert_eq!(
            parsed,
            NodeBinLaunch {
                script_path: script.canonicalize().unwrap(),
                node_args: Vec::new(),
            }
        );
    }

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
}
