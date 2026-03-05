use std::{env, ffi::OsStr, path::PathBuf, process::ExitCode};

use anyhow::{anyhow, Result};

use crate::{
    app::{completion::print_completion, doctor::print_doctor},
    core::{
        config::HniConfig,
        detect::detect,
        resolve::ResolveContext,
        runner,
        types::{InvocationKind, ResolvedExecution},
    },
    features,
};

use super::{flags::parse_global_flags, help::print_help, version::print_versions};

pub fn run_from_env() -> Result<ExitCode> {
    let mut argv = env::args().collect::<Vec<_>>();
    let argv0 = argv
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("missing argv[0]"))?;
    let mut invocation = invocation_from_argv0(&argv0);

    argv.remove(0);

    let cwd = env::current_dir()?;
    let parsed = parse_global_flags(&cwd, argv)?;

    if !parsed.cwd.exists() {
        return Err(anyhow!(
            "working directory does not exist: {}",
            parsed.cwd.display()
        ));
    }

    let config = HniConfig::load()?;
    let resolve_ctx = ResolveContext::new(parsed.cwd.clone(), config.clone());

    if parsed.show_help {
        print_help(invocation);
        return Ok(ExitCode::SUCCESS);
    }

    if parsed.show_version {
        print_versions(&resolve_ctx);
        return Ok(ExitCode::SUCCESS);
    }

    let mut command_args = parsed.args;

    if invocation == InvocationKind::Hni {
        if command_args.is_empty() {
            print_help(invocation);
            return Ok(ExitCode::SUCCESS);
        }

        if handle_hni_meta_subcommand(&command_args, &resolve_ctx, &argv0)? {
            return Ok(ExitCode::SUCCESS);
        }

        if let Some(mapped) = invocation_from_hni_subcommand(&command_args[0]) {
            invocation = mapped;
            command_args = command_args.into_iter().skip(1).collect();
        } else {
            return Err(anyhow!(
                "unknown hni command '{}'. Try: hni -h",
                command_args[0]
            ));
        }
    }

    if parsed.explain {
        let resolved = dispatch_invocation(invocation, command_args, &resolve_ctx)?;
        let Some(resolved) = resolved else {
            return Ok(ExitCode::SUCCESS);
        };
        print_explain(invocation, &resolved, &resolve_ctx, &config)?;
        return Ok(ExitCode::SUCCESS);
    }

    let resolved = dispatch_invocation(invocation, command_args, &resolve_ctx)?;
    let Some(resolved) = resolved else {
        return Ok(ExitCode::SUCCESS);
    };

    if parsed.debug {
        println!("{}", runner::format_debug(&resolved, config.use_sfw)?);
        return Ok(ExitCode::SUCCESS);
    }

    runner::run(&resolved, config.use_sfw)
}

fn handle_hni_meta_subcommand(args: &[String], ctx: &ResolveContext, argv0: &str) -> Result<bool> {
    let Some((first, rest)) = args.split_first() else {
        return Ok(false);
    };

    match first.as_str() {
        "doctor" => {
            print_doctor(&ctx.cwd, &ctx.config);
            Ok(true)
        }
        "help" => {
            print_help(InvocationKind::Hni);
            Ok(true)
        }
        "completion" => {
            let shell = rest.first().map(String::as_str);
            let program = normalized_program_name(argv0);
            print_completion(shell, &program)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn invocation_from_hni_subcommand(value: &str) -> Option<InvocationKind> {
    match value {
        "ni" => Some(InvocationKind::Ni),
        "nr" => Some(InvocationKind::Nr),
        "nlx" => Some(InvocationKind::Nlx),
        "nu" => Some(InvocationKind::Nu),
        "nun" => Some(InvocationKind::Nun),
        "nci" => Some(InvocationKind::Nci),
        "na" => Some(InvocationKind::Na),
        "np" => Some(InvocationKind::Np),
        "ns" => Some(InvocationKind::Ns),
        "node" => Some(InvocationKind::NodeShim),
        _ => None,
    }
}

fn print_explain(
    invocation: InvocationKind,
    resolved: &ResolvedExecution,
    ctx: &ResolveContext,
    config: &HniConfig,
) -> Result<()> {
    println!("hni explain");
    println!("invocation: {}", invocation_name(invocation));
    println!("cwd: {}", ctx.cwd.display());
    println!(
        "resolved: {}",
        runner::format_debug(resolved, config.use_sfw)?
    );

    if let Ok(detection) = detect(&ctx.cwd, &ctx.config) {
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
) -> Result<Option<ResolvedExecution>> {
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
        InvocationKind::Hni => Ok(None),
    }
}

fn invocation_from_argv0(argv0: &str) -> InvocationKind {
    match normalized_program_name(argv0).as_str() {
        "ni" => InvocationKind::Ni,
        "nr" => InvocationKind::Nr,
        "nlx" => InvocationKind::Nlx,
        "nu" => InvocationKind::Nu,
        "nun" => InvocationKind::Nun,
        "nci" => InvocationKind::Nci,
        "na" => InvocationKind::Na,
        "np" => InvocationKind::Np,
        "ns" => InvocationKind::Ns,
        "node" => InvocationKind::NodeShim,
        _ => InvocationKind::Hni,
    }
}

fn normalized_program_name(argv0: &str) -> String {
    let name = PathBuf::from(argv0)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(argv0)
        .to_ascii_lowercase();
    name.strip_suffix(".exe").unwrap_or(&name).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_node_shim_from_argv() {
        assert_eq!(
            invocation_from_argv0("/usr/local/bin/node"),
            InvocationKind::NodeShim
        );
    }

    #[test]
    fn detects_np_and_ns_from_argv() {
        assert_eq!(
            invocation_from_argv0("/usr/local/bin/np"),
            InvocationKind::Np
        );
        assert_eq!(
            invocation_from_argv0("/usr/local/bin/ns"),
            InvocationKind::Ns
        );
    }
}
