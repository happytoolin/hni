use std::process::ExitCode;

use crate::{
    app::{
        cli::{ParsedCommand, parse_from_env},
        completion::print_completion,
        doctor::print_doctor,
        help::print_help,
        init::print_init,
        version::print_versions,
    },
    core::{
        config::{HniConfig, RunAgent},
        detect::detect,
        error::{HniError, HniResult},
        resolve::ResolveContext,
        runner,
        types::{ExecutionMode, InvocationKind, ResolvedExecution},
    },
    features,
    platform::node::resolve_real_node_path,
};

/// Run the application based on command-line arguments.
///
/// # Errors
///
/// Returns an error if:
/// - Argument parsing fails
/// - Working directory does not exist
/// - Configuration loading fails
/// - Command execution fails
pub fn run_from_env() -> HniResult<ExitCode> {
    let parsed = parse_from_env()?;
    if parsed.deprecated_debug_alias_used {
        eprintln!(
            "[hni] warning: '?' debug alias is deprecated; use --debug-resolved, --dry-run, or --print-command"
        );
    }

    if !parsed.cwd.exists() {
        return Err(HniError::execution(format!(
            "working directory does not exist: {}",
            parsed.cwd.display()
        )));
    }

    let mut config = HniConfig::load()?;
    if let Some(native_override) = parsed.native_override {
        config.native_mode = native_override;
        if !native_override {
            config.run_agent = RunAgent::PackageManager;
        }
    }
    let verify_package_manager_availability =
        matches!(&parsed.command, ParsedCommand::Execute { .. })
            && !parsed.debug
            && !parsed.explain;
    let resolve_ctx = ResolveContext::with_package_manager_checks(
        parsed.cwd.clone(),
        config.clone(),
        verify_package_manager_availability,
    );

    match parsed.command {
        ParsedCommand::PrintVersions => {
            print_versions(&resolve_ctx);
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::PrintHelp(invocation) => {
            print_help(invocation);
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::Doctor => {
            print_doctor(&resolve_ctx);
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::Completion { shell, program } => {
            print_completion(shell.as_deref(), &program)?;
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::Init { shell } => {
            print_init(&shell)?;
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::InternalRealNodePath => {
            if let Ok(path) = resolve_real_node_path() {
                println!("{}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::InternalProfileLoop {
            invocation,
            args,
            iterations,
        } => {
            run_profile_loop(invocation, args, iterations, &resolve_ctx, config.use_sfw)?;
            Ok(ExitCode::SUCCESS)
        }
        ParsedCommand::Execute { invocation, args } => {
            let resolved = dispatch_invocation(invocation, args, &resolve_ctx)?;
            let Some(resolved) = resolved else {
                return Ok(ExitCode::SUCCESS);
            };

            if parsed.explain {
                print_explain(invocation, &resolved, &resolve_ctx, &config)?;
                return Ok(ExitCode::SUCCESS);
            }

            if parsed.debug {
                let debug_rendered = runner::format_debug(&resolved, config.use_sfw)
                    .map_err(|error| HniError::execution(error.to_string()))?;
                println!("{debug_rendered}");
                return Ok(ExitCode::SUCCESS);
            }

            runner::run(&resolved, config.use_sfw)
                .map_err(|error| HniError::execution(error.to_string()))
        }
    }
}

fn print_explain(
    invocation: InvocationKind,
    resolved: &ResolvedExecution,
    ctx: &ResolveContext,
    config: &HniConfig,
) -> HniResult<()> {
    println!("hni explain");
    println!("invocation: {}", invocation_name(invocation));
    println!("cwd: {}", ctx.cwd().display());
    println!("native_mode: {}", config.native_mode);
    println!("execution_mode: {}", resolved.execution_mode_name());
    if config.native_mode {
        let native_status = if resolved.native_fallback_reason.is_some() {
            "fallback"
        } else if matches!(
            resolved.mode,
            ExecutionMode::Native | ExecutionMode::NodeRun
        ) {
            "eligible"
        } else {
            "not-applicable"
        };
        println!("native_status: {}", native_status);
        if let Some(reason) = &resolved.native_fallback_reason {
            println!("native_fallback_reason: {reason}");
        }
    }
    println!(
        "resolved: {}",
        runner::format_debug(resolved, config.use_sfw)
            .map_err(|error| HniError::execution(error.to_string()))?
    );

    if let Ok(detection) = ctx.detect().or_else(|_| detect(ctx.cwd(), &ctx.config)) {
        println!(
            "detected_agent: {}",
            detection
                .agent
                .map_or_else(|| "none".to_string(), |pm| pm.display_name().to_string())
        );
        println!("detection_source: {:?}", detection.source);
        println!("has_lockfile: {}", detection.has_lock);
    }

    Ok(())
}

fn run_profile_loop(
    invocation: InvocationKind,
    args: Vec<String>,
    iterations: usize,
    ctx: &ResolveContext,
    use_sfw: bool,
) -> HniResult<()> {
    for _ in 0..iterations {
        let resolved = dispatch_invocation(invocation, args.clone(), ctx)?;
        if let Some(resolved) = resolved {
            std::hint::black_box(
                runner::format_debug(&resolved, use_sfw)
                    .map_err(|error| HniError::execution(error.to_string()))?,
            );
        }
    }

    Ok(())
}

fn invocation_name(invocation: InvocationKind) -> &'static str {
    match invocation {
        InvocationKind::Hni => "hni",
        InvocationKind::Ni => "ni",
        InvocationKind::Nr => "nr",
        InvocationKind::Nlx => "nlx",
        InvocationKind::Nu => "nu",
        InvocationKind::Nun => "nun",
        InvocationKind::Nci => "nci",
        InvocationKind::Na => "na",
        InvocationKind::Np => "np",
        InvocationKind::Ns => "ns",
        InvocationKind::NodeShim => "node",
    }
}

fn dispatch_invocation(
    invocation: InvocationKind,
    args: Vec<String>,
    ctx: &ResolveContext,
) -> HniResult<Option<ResolvedExecution>> {
    match invocation {
        InvocationKind::Ni => features::ni::handle(args, ctx),
        InvocationKind::Nr => features::nr::handle(args, ctx),
        InvocationKind::Nlx => features::nlx::handle(args, ctx),
        InvocationKind::Nu => features::nu::handle(args, ctx),
        InvocationKind::Nun => features::nun::handle(args, ctx),
        InvocationKind::Nci => features::nci::handle(args, ctx),
        InvocationKind::Na => features::na::handle(args, ctx),
        InvocationKind::Np => features::np::handle(args, ctx),
        InvocationKind::Ns => features::ns::handle(args, ctx),
        InvocationKind::NodeShim => features::node_shim::handle(args, ctx),
        InvocationKind::Hni => Err(HniError::execution("missing command")),
    }
}
