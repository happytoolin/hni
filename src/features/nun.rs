use anyhow::Result;

use crate::{
    core::{resolve, resolve::ResolveContext, types::ResolvedExecution},
    features::interactive::nun_select::choose_dependencies_for_uninstall,
};

pub fn handle(mut args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    let interactive_multi = args.iter().any(|arg| arg == "-m");
    if interactive_multi {
        args.retain(|arg| arg != "-m");
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
