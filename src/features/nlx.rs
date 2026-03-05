use anyhow::{anyhow, Result};

use crate::core::{resolve, resolve::ResolveContext, types::ResolvedExecution};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    if args.is_empty() {
        return Err(anyhow!("nlx requires a command to execute"));
    }

    Ok(Some(resolve::resolve_nlx(args, ctx)?))
}
