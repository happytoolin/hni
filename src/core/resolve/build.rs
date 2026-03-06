use std::path::Path;

use crate::core::{
    config::RunAgent,
    error::{HniError, HniResult},
    types::{Intent, PackageManager, ResolvedExecution},
};

use super::{
    context::ResolveContext,
    detect::detect_for_action,
    flags::{exclude_flag, normalize_ni_args, prepend},
    map::{
        add_command, execute_command, frozen_command, global_install_command,
        global_uninstall_command, install_command, run_command, uninstall_command, upgrade_command,
    },
};

pub fn resolve_ni(args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let use_global = args.iter().any(|arg| arg == "-g");
    let detected = detect_for_action(&ctx.cwd, &ctx.config, use_global)?;
    let args = normalize_ni_args(args, detected.pm);

    if use_global {
        let args = exclude_flag(args, "-g");
        return Ok(build_exec(
            detected.pm,
            Intent::Install,
            args,
            &ctx.cwd,
            true,
            detected.has_lock,
        ));
    }

    if args.iter().any(|a| a == "--frozen-if-present") {
        let args = exclude_flag(args, "--frozen-if-present");
        if detected.has_lock {
            return Ok(build_exec(
                detected.pm,
                Intent::CleanInstall,
                args,
                &ctx.cwd,
                false,
                true,
            ));
        }
        return Ok(build_exec(
            detected.pm,
            Intent::Install,
            args,
            &ctx.cwd,
            false,
            false,
        ));
    }

    if args.iter().any(|a| a == "--frozen") {
        let args = exclude_flag(args, "--frozen");
        return Ok(build_exec(
            detected.pm,
            Intent::CleanInstall,
            args,
            &ctx.cwd,
            false,
            true,
        ));
    }

    if args.is_empty() || args.iter().all(|a| a.starts_with('-')) {
        return Ok(build_exec(
            detected.pm,
            Intent::Install,
            args,
            &ctx.cwd,
            false,
            detected.has_lock,
        ));
    }

    Ok(build_exec(
        detected.pm,
        Intent::Add,
        args,
        &ctx.cwd,
        false,
        detected.has_lock,
    ))
}

pub fn resolve_nr(mut args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;

    if args.is_empty() {
        args.push("start".to_string());
    }

    let has_if_present = args.iter().any(|a| a == "--if-present");
    if has_if_present {
        args = exclude_flag(args, "--if-present");
    }

    if ctx.config.run_agent == RunAgent::Node {
        let mut node_args = vec!["--run".to_string()];
        node_args.extend(args);

        return Ok(ResolvedExecution {
            program: "node".to_string(),
            args: node_args,
            cwd: ctx.cwd.clone(),
            passthrough: true,
        });
    }

    let mut resolved = build_exec(
        detected.pm,
        Intent::Run,
        args,
        &ctx.cwd,
        false,
        detected.has_lock,
    );

    if has_if_present {
        if let Some(first) = resolved.args.first() {
            if matches!(first.as_str(), "run" | "task") {
                resolved.args.insert(1, "--if-present".to_string());
            } else {
                resolved.args.insert(0, "--if-present".to_string());
            }
        } else {
            resolved.args.push("--if-present".to_string());
        }
    }

    Ok(resolved)
}

pub fn resolve_nlx(args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;
    Ok(build_exec(
        detected.pm,
        Intent::Execute,
        args,
        &ctx.cwd,
        false,
        detected.has_lock,
    ))
}

pub fn resolve_nu(mut args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;
    let interactive = args
        .iter()
        .any(|a| matches!(a.as_str(), "-i" | "--interactive"));
    if interactive {
        args = exclude_flag(args, "-i");
        args = exclude_flag(args, "--interactive");
    }

    build_upgrade_exec(detected.pm, args, &ctx.cwd, interactive)
}

pub fn resolve_nun(args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let use_global = args.iter().any(|arg| arg == "-g");
    let detected = detect_for_action(&ctx.cwd, &ctx.config, use_global)?;
    let args = if use_global {
        exclude_flag(args, "-g")
    } else {
        args
    };

    if args.is_empty() {
        return Err(HniError::execution(
            "no dependencies selected for uninstall",
        ));
    }

    Ok(build_exec(
        detected.pm,
        Intent::Uninstall,
        args,
        &ctx.cwd,
        use_global,
        detected.has_lock,
    ))
}

pub fn resolve_nci(args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;

    if detected.has_lock {
        Ok(build_exec(
            detected.pm,
            Intent::CleanInstall,
            args,
            &ctx.cwd,
            false,
            true,
        ))
    } else {
        Ok(build_exec(
            detected.pm,
            Intent::Install,
            args,
            &ctx.cwd,
            false,
            false,
        ))
    }
}

pub fn resolve_na(args: Vec<String>, ctx: &ResolveContext) -> HniResult<ResolvedExecution> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;
    Ok(build_exec(
        detected.pm,
        Intent::AgentAlias,
        args,
        &ctx.cwd,
        false,
        detected.has_lock,
    ))
}

pub fn resolve_node_passthrough(args: Vec<String>, cwd: &Path) -> ResolvedExecution {
    ResolvedExecution {
        program: "node".to_string(),
        args,
        cwd: cwd.to_path_buf(),
        passthrough: true,
    }
}

pub fn resolve_node_routed(
    intent: Intent,
    args: Vec<String>,
    ctx: &ResolveContext,
) -> HniResult<ResolvedExecution> {
    match intent {
        Intent::Install => resolve_ni(args, ctx),
        Intent::Add | Intent::Execute => resolve_detected_intent(intent, args, ctx),
        Intent::Run => resolve_nr(args, ctx),
        Intent::Upgrade => resolve_nu(args, ctx),
        Intent::Uninstall => resolve_nun(args, ctx),
        Intent::CleanInstall => resolve_nci(args, ctx),
        Intent::AgentAlias => resolve_na(args, ctx),
        Intent::PassthroughNode => Ok(resolve_node_passthrough(args, &ctx.cwd)),
    }
}

fn resolve_detected_intent(
    intent: Intent,
    args: Vec<String>,
    ctx: &ResolveContext,
) -> HniResult<ResolvedExecution> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;
    Ok(build_exec(
        detected.pm,
        intent,
        args,
        &ctx.cwd,
        false,
        detected.has_lock,
    ))
}

fn build_upgrade_exec(
    pm: PackageManager,
    args: Vec<String>,
    cwd: &Path,
    interactive: bool,
) -> HniResult<ResolvedExecution> {
    let (program, args) = if interactive {
        match pm {
            PackageManager::Npm | PackageManager::Bun => {
                return Err(HniError::interactive(format!(
                    "interactive upgrade is not supported for {}",
                    pm.display_name()
                )));
            }
            PackageManager::Yarn => ("yarn".to_string(), prepend("upgrade-interactive", args)),
            PackageManager::YarnBerry => ("yarn".to_string(), prepend("up", prepend("-i", args))),
            PackageManager::Pnpm => ("pnpm".to_string(), prepend("update", prepend("-i", args))),
            PackageManager::Deno => (
                "deno".to_string(),
                prepend("outdated", prepend("--update", args)),
            ),
        }
    } else {
        upgrade_command(pm, args)
    };

    Ok(ResolvedExecution {
        program,
        args,
        cwd: cwd.to_path_buf(),
        passthrough: false,
    })
}

fn build_exec(
    pm: PackageManager,
    intent: Intent,
    args: Vec<String>,
    cwd: &Path,
    use_global: bool,
    has_lock: bool,
) -> ResolvedExecution {
    let (program, args) = match intent {
        Intent::Install => {
            if use_global {
                global_install_command(pm, args)
            } else {
                install_command(pm, args)
            }
        }
        Intent::Add => add_command(pm, args),
        Intent::Run => run_command(pm, args),
        Intent::Execute => execute_command(pm, args),
        Intent::Upgrade => upgrade_command(pm, args),
        Intent::Uninstall => {
            if use_global {
                global_uninstall_command(pm, args)
            } else {
                uninstall_command(pm, args)
            }
        }
        Intent::CleanInstall => {
            if has_lock {
                frozen_command(pm)
            } else {
                install_command(pm, args)
            }
        }
        Intent::AgentAlias => (pm.bin().to_string(), args),
        Intent::PassthroughNode => {
            return ResolvedExecution {
                program: "node".to_string(),
                args,
                cwd: cwd.to_path_buf(),
                passthrough: true,
            };
        }
    };

    ResolvedExecution {
        program,
        args,
        cwd: cwd.to_path_buf(),
        passthrough: false,
    }
}
