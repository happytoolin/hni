use anyhow::Result;

use crate::{
    core::{resolve, resolve::ResolveContext, types::ResolvedExecution},
    features::interactive::nun_select::choose_dependencies_for_uninstall,
};

pub fn handle(mut args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    let interactive_multi = args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-m" | "--multi-select"));
    if interactive_multi {
        args = crate::core::resolve::exclude_flag(args, "-m");
        args = crate::core::resolve::exclude_flag(args, "--multi-select");
    }

    if args.is_empty() || interactive_multi {
        let selected = choose_dependencies_for_uninstall(&ctx.cwd)?;
        if selected.is_empty() {
            return Ok(None);
        }

        args.extend(selected);
    }

    Ok(Some(resolve::resolve_nun(args, ctx)?))
}
