use crate::{
    core::{error::HniResult, resolve, resolve::ResolveContext, types::ResolvedExecution},
    features::interactive::ni_search::augment_ni_args_interactive,
};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> HniResult<Option<ResolvedExecution>> {
    let args = augment_ni_args_interactive(args, ctx)?;
    let resolved = resolve::resolve_ni(args, ctx)?;
    Ok(Some(resolved))
}
