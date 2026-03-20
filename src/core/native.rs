use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use anyhow::{Context, Result};

use crate::{
    core::{
        error::HniResult,
        package::{
            find_nearest_package, node_modules_bin_dirs, resolve_declared_package_bin,
            resolve_local_bin,
        },
        resolve::ResolveContext,
        shell::shell_escape,
        types::{
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
    pm: PackageManager,
    args: &[String],
    ctx: &ResolveContext,
    has_if_present: bool,
) -> HniResult<NativeAttempt> {
    if pm == PackageManager::Deno {
        return Ok(NativeAttempt::Ineligible(
            "deno script execution stays delegated".to_string(),
        ));
    }

    let Some(pkg) = find_nearest_package(&ctx.cwd)? else {
        return Ok(NativeAttempt::Ineligible(
            "native script execution requires a nearest package.json".to_string(),
        ));
    };

    let scripts = pkg.manifest.scripts.unwrap_or_default();
    let bin_paths = node_modules_bin_dirs(&ctx.cwd);

    if pm == PackageManager::YarnBerry && has_yarn_pnp_loader(&ctx.cwd) {
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
                    ctx.cwd.clone(),
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
        ResolvedExecution::native_script(script_name, ctx.cwd.clone(), exec),
    )))
}

pub fn attempt_nlx(
    pm: PackageManager,
    args: &[String],
    ctx: &ResolveContext,
) -> HniResult<NativeAttempt> {
    if pm == PackageManager::Deno {
        return Ok(NativeAttempt::Ineligible(
            "deno exec stays delegated".to_string(),
        ));
    }

    let Some(bin_name) = args.first() else {
        return Ok(NativeAttempt::Ineligible(
            "native local bin execution requires a command".to_string(),
        ));
    };
    if pm == PackageManager::YarnBerry && has_yarn_pnp_loader(&ctx.cwd) {
        return Ok(NativeAttempt::Ineligible(
            "yarn berry Plug'n'Play does not expose node_modules/.bin; falling back to yarn execution"
                .to_string(),
        ));
    }
    let bin_paths = node_modules_bin_dirs(&ctx.cwd);
    let bin_path = resolve_local_bin(bin_name, &bin_paths).or_else(|| {
        resolve_declared_package_bin(&ctx.cwd, bin_name)
            .ok()
            .flatten()
    });
    let Some(bin_path) = bin_path else {
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
            ctx.cwd.clone(),
            exec,
        ),
    )))
}

pub fn run_script(exec: &NativeScriptExecution, invocation_cwd: &Path) -> Result<ExitCode> {
    if exec.steps.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }

    let shared_env = native_script_env(exec, invocation_cwd)?;

    for step in &exec.steps {
        let forwarded_args = if step.forward_args {
            exec.forwarded_args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let status = shell_command(&step.command, &forwarded_args, &exec.package_root)
            .envs(shared_env.clone())
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

fn has_yarn_pnp_loader(start: &Path) -> bool {
    start
        .ancestors()
        .any(|dir| dir.join(".pnp.cjs").exists() || dir.join(".pnp.js").exists())
}

fn native_script_env(
    exec: &NativeScriptExecution,
    invocation_cwd: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut envs = BTreeMap::new();
    envs.insert(
        "INIT_CWD".to_string(),
        invocation_cwd.to_string_lossy().to_string(),
    );
    envs.insert(
        "npm_package_json".to_string(),
        exec.package_json_path.to_string_lossy().to_string(),
    );

    if let Ok(current_exe) = env::current_exe() {
        envs.insert(
            "npm_execpath".to_string(),
            current_exe.to_string_lossy().to_string(),
        );
    }

    if let Ok(real_node) = resolve_real_node_path() {
        envs.insert(
            "npm_node_execpath".to_string(),
            real_node.to_string_lossy().to_string(),
        );
    }

    let merged_path = merged_path_with_bins(&exec.bin_paths)?;
    envs.insert("PATH".to_string(), merged_path);
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

fn shell_command(command_string: &str, forwarded_args: &[&str], cwd: &Path) -> Command {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.arg("/C").arg(command_string);
        command
    } else {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(format!("{command_string} \"$@\""))
            .arg("sh");
        command
    };

    command
        .args(forwarded_args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
}

fn exit_code_from_status(code: Option<i32>) -> ExitCode {
    code.map_or_else(|| ExitCode::from(1), exit_code_from_code)
}

fn exit_code_from_code(code: i32) -> ExitCode {
    let code = u8::try_from(code).unwrap_or(1);
    ExitCode::from(code)
}
