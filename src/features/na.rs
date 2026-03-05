use anyhow::Result;

use crate::core::{resolve, resolve::ResolveContext, types::ResolvedExecution};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    if args.is_empty() {
        let pm = resolve::detected_package_manager(ctx)?;
        println!("{}", pm.display_name());
        return Ok(None);
    }

    Ok(Some(resolve::resolve_na(args, ctx)?))
}
