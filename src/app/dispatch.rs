use std::{env, ffi::OsStr, path::PathBuf, process::ExitCode};

use anyhow::{anyhow, Result};

use crate::{
    core::{
        config::HniConfig,
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
    let invocation = invocation_from_argv0(&argv0);

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

    if invocation == InvocationKind::Hni {
        print_help(invocation);
        return Ok(ExitCode::SUCCESS);
    }

    let resolved = dispatch_invocation(invocation, parsed.args, &resolve_ctx)?;
    let Some(resolved) = resolved else {
        return Ok(ExitCode::SUCCESS);
    };

    if parsed.debug {
        println!("{}", runner::format_debug(&resolved, config.use_sfw)?);
        return Ok(ExitCode::SUCCESS);
    }

    runner::run(&resolved, config.use_sfw)
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
    let name = PathBuf::from(argv0)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(argv0)
        .to_ascii_lowercase();
    let normalized = name.strip_suffix(".exe").unwrap_or(&name);

    match normalized {
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
