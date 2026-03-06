use crate::core::{
    error::{HniError, HniResult},
    resolve,
    resolve::ResolveContext,
    types::ResolvedExecution,
};

pub fn handle(args: Vec<String>, ctx: &ResolveContext) -> HniResult<Option<ResolvedExecution>> {
    if args.is_empty() {
        return Err(HniError::execution(
            "nlx requires a command to execute.\nTry: nlx create-vite@latest",
        ));
    }

    Ok(Some(resolve::resolve_nlx(args, ctx)?))
}
