use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::{
    config::{HniConfig, RunAgent},
    detect::{detect, ensure_package_manager_available},
    types::{DetectionResult, Intent, PackageManager, ResolvedExecution},
};

#[derive(Debug, Clone)]
pub struct ResolveContext {
    pub cwd: PathBuf,
    pub config: HniConfig,
}

impl ResolveContext {
    pub fn new(cwd: PathBuf, config: HniConfig) -> Self {
        Self { cwd, config }
    }
}

pub fn resolve_ni(args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
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

pub fn resolve_nr(mut args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
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

pub fn resolve_nlx(args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
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

pub fn resolve_nu(mut args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
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

pub fn resolve_nun(args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
    let use_global = args.iter().any(|arg| arg == "-g");
    let detected = detect_for_action(&ctx.cwd, &ctx.config, use_global)?;
    let args = if use_global {
        exclude_flag(args, "-g")
    } else {
        args
    };

    if args.is_empty() {
        return Err(anyhow!("no dependencies selected for uninstall"));
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

pub fn resolve_nci(args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
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

pub fn resolve_na(args: Vec<String>, ctx: &ResolveContext) -> Result<ResolvedExecution> {
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
) -> Result<ResolvedExecution> {
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

pub fn detected_package_manager(ctx: &ResolveContext) -> Result<PackageManager> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;
    Ok(detected.pm)
}

fn resolve_detected_intent(
    intent: Intent,
    args: Vec<String>,
    ctx: &ResolveContext,
) -> Result<ResolvedExecution> {
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

fn detect_for_action(cwd: &Path, config: &HniConfig, use_global: bool) -> Result<AgentResolution> {
    let detection = if use_global {
        DetectionResult {
            agent: Some(config.global_agent),
            has_lock: false,
            version_hint: None,
            source: super::types::DetectionSource::Config,
        }
    } else {
        detect(cwd, config)?
    };

    let pm = detection
        .agent
        .ok_or_else(|| {
            anyhow!(
                "unable to detect package manager in {}.\nAdd packageManager to package.json, add a lockfile, or set defaultAgent in ~/.hnirc",
                cwd.display()
            )
        })?;

    if use_global && pm == PackageManager::YarnBerry {
        return Err(anyhow!(
            "global install/uninstall is not supported by yarn (berry).\nUse a different globalAgent (for example: npm, pnpm, yarn, bun, deno)."
        ));
    }

    ensure_package_manager_available(pm, detection.version_hint.as_deref(), config, cwd)?;

    Ok(AgentResolution {
        pm,
        has_lock: detection.has_lock,
    })
}

fn build_upgrade_exec(
    pm: PackageManager,
    args: Vec<String>,
    cwd: &Path,
    interactive: bool,
) -> Result<ResolvedExecution> {
    let (program, args) = if interactive {
        match pm {
            PackageManager::Npm | PackageManager::Bun => {
                return Err(anyhow!(
                    "interactive upgrade is not supported for {}",
                    pm.display_name()
                ));
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

pub fn version_command_for_pm(pm: PackageManager) -> (String, Vec<String>) {
    (pm.bin().to_string(), vec!["--version".to_string()])
}

fn install_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), prepend("i", args)),
        PackageManager::Yarn | PackageManager::YarnBerry => {
            ("yarn".to_string(), prepend("install", args))
        }
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("i", args)),
        PackageManager::Bun => ("bun".to_string(), prepend("install", args)),
        PackageManager::Deno => ("deno".to_string(), prepend("install", args)),
    }
}

fn add_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), prepend("i", args)),
        PackageManager::Yarn | PackageManager::YarnBerry => {
            ("yarn".to_string(), prepend("add", args))
        }
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("add", args)),
        PackageManager::Bun => ("bun".to_string(), prepend("add", args)),
        PackageManager::Deno => ("deno".to_string(), prepend("add", args)),
    }
}

fn run_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), npm_run_args(args)),
        PackageManager::Yarn | PackageManager::YarnBerry => {
            ("yarn".to_string(), prepend("run", args))
        }
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("run", args)),
        PackageManager::Bun => ("bun".to_string(), prepend("run", args)),
        PackageManager::Deno => ("deno".to_string(), prepend("task", args)),
    }
}

fn execute_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm | PackageManager::Yarn => ("npx".to_string(), args),
        PackageManager::YarnBerry => ("yarn".to_string(), prepend("dlx", args)),
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("dlx", args)),
        PackageManager::Bun => ("bun".to_string(), prepend("x", args)),
        PackageManager::Deno => {
            if let Some((first, rest)) = args.split_first() {
                let mut deno_args = vec!["run".to_string(), format!("npm:{first}")];
                deno_args.extend(rest.iter().cloned());
                ("deno".to_string(), deno_args)
            } else {
                ("deno".to_string(), vec!["run".to_string()])
            }
        }
    }
}

fn upgrade_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), prepend("update", args)),
        PackageManager::Yarn => ("yarn".to_string(), prepend("upgrade", args)),
        PackageManager::YarnBerry => ("yarn".to_string(), prepend("up", args)),
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("update", args)),
        PackageManager::Bun => ("bun".to_string(), prepend("update", args)),
        PackageManager::Deno => (
            "deno".to_string(),
            prepend("outdated", prepend("--update", args)),
        ),
    }
}

fn uninstall_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), prepend("uninstall", args)),
        PackageManager::Yarn | PackageManager::YarnBerry => {
            ("yarn".to_string(), prepend("remove", args))
        }
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("remove", args)),
        PackageManager::Bun => ("bun".to_string(), prepend("remove", args)),
        PackageManager::Deno => ("deno".to_string(), prepend("remove", args)),
    }
}

fn frozen_command(pm: PackageManager) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), vec!["ci".to_string()]),
        PackageManager::Yarn => (
            "yarn".to_string(),
            vec!["install".to_string(), "--frozen-lockfile".to_string()],
        ),
        PackageManager::YarnBerry => (
            "yarn".to_string(),
            vec!["install".to_string(), "--immutable".to_string()],
        ),
        PackageManager::Pnpm => (
            "pnpm".to_string(),
            vec!["i".to_string(), "--frozen-lockfile".to_string()],
        ),
        PackageManager::Bun => (
            "bun".to_string(),
            vec!["install".to_string(), "--frozen-lockfile".to_string()],
        ),
        PackageManager::Deno => (
            "deno".to_string(),
            vec!["install".to_string(), "--frozen".to_string()],
        ),
    }
}

fn global_install_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), prepend("i", prepend("-g", args))),
        PackageManager::Yarn | PackageManager::YarnBerry => yarn_global_command("add", args),
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("add", prepend("-g", args))),
        PackageManager::Bun => ("bun".to_string(), prepend("add", prepend("-g", args))),
        PackageManager::Deno => ("deno".to_string(), prepend("install", prepend("-g", args))),
    }
}

fn global_uninstall_command(pm: PackageManager, args: Vec<String>) -> (String, Vec<String>) {
    match pm {
        PackageManager::Npm => ("npm".to_string(), prepend("uninstall", prepend("-g", args))),
        PackageManager::Yarn | PackageManager::YarnBerry => yarn_global_command("remove", args),
        PackageManager::Pnpm => ("pnpm".to_string(), prepend("remove", prepend("-g", args))),
        PackageManager::Bun => ("bun".to_string(), prepend("remove", prepend("-g", args))),
        PackageManager::Deno => (
            "deno".to_string(),
            prepend("uninstall", prepend("-g", args)),
        ),
    }
}

fn yarn_global_command(action: &str, args: Vec<String>) -> (String, Vec<String>) {
    let mut out = vec!["global".to_string(), action.to_string()];
    out.extend(args);
    ("yarn".to_string(), out)
}

fn npm_run_args(args: Vec<String>) -> Vec<String> {
    if args.len() <= 1 {
        return prepend("run", args);
    }

    let mut out = vec!["run".to_string(), args[0].clone(), "--".to_string()];
    out.extend(args.into_iter().skip(1));
    out
}

fn prepend(head: &str, mut tail: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(tail.len() + 1);
    out.push(head.to_string());
    out.append(&mut tail);
    out
}

pub fn exclude_flag(args: Vec<String>, flag: &str) -> Vec<String> {
    args.into_iter().filter(|arg| arg != flag).collect()
}

fn normalize_ni_args(args: Vec<String>, pm: PackageManager) -> Vec<String> {
    args.into_iter()
        .map(|arg| match arg.as_str() {
            "-D" if pm == PackageManager::Bun => "-d".to_string(),
            "-P" if pm == PackageManager::Npm => "--omit=dev".to_string(),
            "-P" => "--production".to_string(),
            _ => arg,
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct AgentResolution {
    pm: PackageManager,
    has_lock: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        config::{DefaultAgent, HniConfig},
        types::PackageManager,
    };

    #[test]
    fn npm_add_resolves_to_install() {
        let (prog, args) = add_command(PackageManager::Npm, vec!["vite".to_string()]);
        assert_eq!(prog, "npm");
        assert_eq!(args, vec!["i", "vite"]);
    }

    #[test]
    fn install_command_maps_all_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["i", "--ignore-scripts"]),
            (
                PackageManager::Yarn,
                "yarn",
                vec!["install", "--immutable-cache"],
            ),
            (
                PackageManager::YarnBerry,
                "yarn",
                vec!["install", "--immutable-cache"],
            ),
            (PackageManager::Pnpm, "pnpm", vec!["i", "--prefer-offline"]),
            (
                PackageManager::Bun,
                "bun",
                vec!["install", "--frozen-lockfile"],
            ),
            (PackageManager::Deno, "deno", vec!["install", "--reload"]),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = install_command(
                pm,
                expected_args
                    .iter()
                    .skip(1)
                    .map(ToString::to_string)
                    .collect(),
            );
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn add_command_maps_all_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["i", "vite"]),
            (PackageManager::Yarn, "yarn", vec!["add", "vite"]),
            (PackageManager::YarnBerry, "yarn", vec!["add", "vite"]),
            (PackageManager::Pnpm, "pnpm", vec!["add", "vite"]),
            (PackageManager::Bun, "bun", vec!["add", "vite"]),
            (PackageManager::Deno, "deno", vec!["add", "vite"]),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = add_command(pm, vec!["vite".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn run_command_maps_all_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["run", "dev"]),
            (PackageManager::Yarn, "yarn", vec!["run", "dev"]),
            (PackageManager::YarnBerry, "yarn", vec!["run", "dev"]),
            (PackageManager::Pnpm, "pnpm", vec!["run", "dev"]),
            (PackageManager::Bun, "bun", vec!["run", "dev"]),
            (PackageManager::Deno, "deno", vec!["task", "dev"]),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = run_command(pm, vec!["dev".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn execute_command_maps_non_deno_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npx", vec!["tsx"]),
            (PackageManager::Yarn, "npx", vec!["tsx"]),
            (PackageManager::YarnBerry, "yarn", vec!["dlx", "tsx"]),
            (PackageManager::Pnpm, "pnpm", vec!["dlx", "tsx"]),
            (PackageManager::Bun, "bun", vec!["x", "tsx"]),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = execute_command(pm, vec!["tsx".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn upgrade_command_maps_all_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["update", "vite"]),
            (PackageManager::Yarn, "yarn", vec!["upgrade", "vite"]),
            (PackageManager::YarnBerry, "yarn", vec!["up", "vite"]),
            (PackageManager::Pnpm, "pnpm", vec!["update", "vite"]),
            (PackageManager::Bun, "bun", vec!["update", "vite"]),
            (
                PackageManager::Deno,
                "deno",
                vec!["outdated", "--update", "vite"],
            ),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = upgrade_command(pm, vec!["vite".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn uninstall_command_maps_all_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["uninstall", "vite"]),
            (PackageManager::Yarn, "yarn", vec!["remove", "vite"]),
            (PackageManager::YarnBerry, "yarn", vec!["remove", "vite"]),
            (PackageManager::Pnpm, "pnpm", vec!["remove", "vite"]),
            (PackageManager::Bun, "bun", vec!["remove", "vite"]),
            (PackageManager::Deno, "deno", vec!["remove", "vite"]),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = uninstall_command(pm, vec!["vite".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn global_install_command_maps_supported_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["i", "-g", "eslint"]),
            (
                PackageManager::Yarn,
                "yarn",
                vec!["global", "add", "eslint"],
            ),
            (PackageManager::Pnpm, "pnpm", vec!["add", "-g", "eslint"]),
            (PackageManager::Bun, "bun", vec!["add", "-g", "eslint"]),
            (
                PackageManager::Deno,
                "deno",
                vec!["install", "-g", "eslint"],
            ),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = global_install_command(pm, vec!["eslint".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn global_uninstall_command_maps_supported_package_managers() {
        let cases = vec![
            (
                PackageManager::Npm,
                "npm",
                vec!["uninstall", "-g", "eslint"],
            ),
            (
                PackageManager::Yarn,
                "yarn",
                vec!["global", "remove", "eslint"],
            ),
            (PackageManager::Pnpm, "pnpm", vec!["remove", "-g", "eslint"]),
            (PackageManager::Bun, "bun", vec!["remove", "-g", "eslint"]),
            (
                PackageManager::Deno,
                "deno",
                vec!["uninstall", "-g", "eslint"],
            ),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = global_uninstall_command(pm, vec!["eslint".to_string()]);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn frozen_for_yarn_berry_uses_immutable() {
        let (prog, args) = frozen_command(PackageManager::YarnBerry);
        assert_eq!(prog, "yarn");
        assert_eq!(args, vec!["install", "--immutable"]);
    }

    #[test]
    fn frozen_command_maps_all_package_managers() {
        let cases = vec![
            (PackageManager::Npm, "npm", vec!["ci"]),
            (
                PackageManager::Yarn,
                "yarn",
                vec!["install", "--frozen-lockfile"],
            ),
            (
                PackageManager::YarnBerry,
                "yarn",
                vec!["install", "--immutable"],
            ),
            (PackageManager::Pnpm, "pnpm", vec!["i", "--frozen-lockfile"]),
            (
                PackageManager::Bun,
                "bun",
                vec!["install", "--frozen-lockfile"],
            ),
            (PackageManager::Deno, "deno", vec!["install", "--frozen"]),
        ];

        for (pm, expected_prog, expected_args) in cases {
            let (prog, args) = frozen_command(pm);
            assert_eq!(prog, expected_prog);
            assert_eq!(
                args,
                expected_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn default_agent_from_config_used_in_detection() {
        let config = HniConfig {
            default_agent: DefaultAgent::Agent(PackageManager::Pnpm),
            ..HniConfig::default()
        };

        let tmp = tempfile::tempdir().unwrap();
        let result = detect_for_action(tmp.path(), &config, false);
        match result {
            Ok(detected) => assert_eq!(detected.pm, PackageManager::Pnpm),
            Err(err) => {
                assert!(which::which("pnpm").is_err());
                assert!(err
                    .to_string()
                    .contains("detected pnpm but it is not installed"));
            }
        }
    }

    #[test]
    fn global_mode_rejects_yarn_berry() {
        let config = HniConfig {
            global_agent: PackageManager::YarnBerry,
            ..HniConfig::default()
        };

        let tmp = tempfile::tempdir().unwrap();
        let err = detect_for_action(tmp.path(), &config, true).unwrap_err();
        assert!(err
            .to_string()
            .contains("global install/uninstall is not supported by yarn (berry)"));
    }

    #[test]
    fn npm_run_inserts_double_dash_for_extra_args() {
        let (_, args) = run_command(
            PackageManager::Npm,
            vec!["dev".to_string(), "--port=3000".to_string()],
        );
        assert_eq!(args, vec!["run", "dev", "--", "--port=3000"]);
    }

    #[test]
    fn npm_run_without_extra_args_avoids_double_dash() {
        let (_, args) = run_command(PackageManager::Npm, vec!["dev".to_string()]);
        assert_eq!(args, vec!["run", "dev"]);
    }

    #[test]
    fn yarn_classic_execute_uses_npx() {
        let (prog, args) = execute_command(PackageManager::Yarn, vec!["vitest".to_string()]);
        assert_eq!(prog, "npx");
        assert_eq!(args, vec!["vitest"]);
    }

    #[test]
    fn bun_execute_uses_x_subcommand() {
        let (prog, args) = execute_command(PackageManager::Bun, vec!["vitest".to_string()]);
        assert_eq!(prog, "bun");
        assert_eq!(args, vec!["x", "vitest"]);
    }

    #[test]
    fn deno_execute_wraps_binary_with_npm_prefix() {
        let (prog, args) = execute_command(
            PackageManager::Deno,
            vec!["vitest".to_string(), "--help".to_string()],
        );
        assert_eq!(prog, "deno");
        assert_eq!(args, vec!["run", "npm:vitest", "--help"]);
    }

    #[test]
    fn deno_execute_without_args_defaults_to_run() {
        let (prog, args) = execute_command(PackageManager::Deno, Vec::new());
        assert_eq!(prog, "deno");
        assert_eq!(args, vec!["run"]);
    }

    #[test]
    fn npm_production_flag_maps_to_omit_dev() {
        let normalized = normalize_ni_args(vec!["-P".to_string()], PackageManager::Npm);
        assert_eq!(normalized, vec!["--omit=dev"]);
    }

    #[test]
    fn non_npm_production_flag_maps_to_production() {
        let normalized = normalize_ni_args(vec!["-P".to_string()], PackageManager::Yarn);
        assert_eq!(normalized, vec!["--production"]);
    }

    #[test]
    fn interactive_upgrade_is_not_supported_for_npm() {
        let tmp = tempfile::tempdir().unwrap();
        let err =
            build_upgrade_exec(PackageManager::Npm, Vec::new(), tmp.path(), true).unwrap_err();
        assert!(err
            .to_string()
            .contains("interactive upgrade is not supported"));
    }
}
