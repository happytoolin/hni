use crate::core::{
    batch::{self, BatchMode},
    error::HniResult,
    resolve::ResolveContext,
    types::ResolvedExecution,
};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> HniResult<Option<ResolvedExecution>> {
    Ok(Some(batch::make_execution(
        BatchMode::Sequential,
        args,
        &ctx.cwd,
    )))
}
