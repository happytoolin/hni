use anyhow::Result;

use crate::core::{
    batch::{self, BatchMode},
    resolve::ResolveContext,
    types::ResolvedExecution,
};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    Ok(Some(batch::make_execution(
        BatchMode::Sequential,
        args,
        &ctx.cwd,
    )))
}
