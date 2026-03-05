use anyhow::Result;

use crate::{
    core::{resolve, resolve::ResolveContext, types::ResolvedExecution},
    features::interactive::ni_search::augment_ni_args_interactive,
};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    let args = augment_ni_args_interactive(args, ctx)?;
    let resolved = resolve::resolve_ni(args, ctx)?;
    Ok(Some(resolved))
}
