use anyhow::Result;

use crate::core::{resolve, resolve::ResolveContext, types::ResolvedExecution};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    Ok(Some(resolve::resolve_nu(args, ctx)?))
}
