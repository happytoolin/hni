use crate::core::{error::HniResult, resolve, resolve::ResolveContext, types::ResolvedExecution};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> HniResult<Option<ResolvedExecution>> {
    Ok(Some(resolve::resolve_nci(args, ctx)?))
}
