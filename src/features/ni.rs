use anyhow::Result;

use crate::{
    core::{resolve, resolve::ResolveContext, types::ResolvedExecution},
    features::interactive::ni_search::augment_ni_args_interactive,
};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    let agent = resolve::detected_package_manager(ctx)?;
    let args = augment_ni_args_interactive(args, agent)?;
    let resolved = resolve::resolve_ni(args, ctx)?;
    Ok(Some(resolved))
}
