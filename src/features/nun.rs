use anyhow::Result;

use crate::{
    core::{resolve, resolve::ResolveContext, types::ResolvedExecution},
    features::interactive::nun_select::choose_dependencies_for_uninstall,
};

pub fn handle(mut args: Vec<String>, ctx: &ResolveContext) -> Result<Option<ResolvedExecution>> {
    let (normalized_args, interactive_multi) = strip_multi_select_flags(args);
    args = normalized_args;

    if args.is_empty() || interactive_multi {
        let selected = choose_dependencies_for_uninstall(&ctx.cwd)?;
        if selected.is_empty() {
            return Ok(None);
        }

        args.extend(selected);
    }

    Ok(Some(resolve::resolve_nun(args, ctx)?))
}

fn strip_multi_select_flags(mut args: Vec<String>) -> (Vec<String>, bool) {
    let interactive_multi = args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-m" | "--multi-select"));
    if interactive_multi {
        args = crate::core::resolve::exclude_flag(args, "-m");
        args = crate::core::resolve::exclude_flag(args, "--multi-select");
    }
    (args, interactive_multi)
}

#[cfg(test)]
mod tests {
    use super::strip_multi_select_flags;

    #[test]
    fn strips_both_multi_select_flag_variants() {
        let (args, interactive) =
            strip_multi_select_flags(vec!["-m".into(), "--multi-select".into(), "vite".into()]);
        assert!(interactive);
        assert_eq!(args, vec!["vite"]);
    }

    #[test]
    fn keeps_args_unchanged_when_multi_select_not_present() {
        let input = vec!["vite".to_string()];
        let (args, interactive) = strip_multi_select_flags(input.clone());
        assert!(!interactive);
        assert_eq!(args, input);
    }
}
