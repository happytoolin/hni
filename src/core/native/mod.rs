mod bin_resolver;
mod eligibility;
mod env;
mod exec;
mod plan;
mod shim_parser;

use std::path::Path;

use anyhow::Result;

use crate::core::{
    resolve::ResolveContext,
    types::{PackageManager, ResolvedExecution},
};

use plan::{NativeDecision, NativePlan};

pub enum NativeAttempt {
    Eligible(Box<ResolvedExecution>),
    Ineligible(String),
}

pub fn attempt_nr(
    pm: Option<PackageManager>,
    args: &[String],
    ctx: &ResolveContext,
    has_if_present: bool,
) -> Result<NativeAttempt> {
    into_attempt(
        eligibility::plan_nr(pm, args, ctx, has_if_present)?,
        ctx.cwd(),
    )
}

pub fn attempt_nlx(
    pm: Option<PackageManager>,
    args: &[String],
    ctx: &ResolveContext,
) -> Result<NativeAttempt> {
    into_attempt(eligibility::plan_nlx(pm, args, ctx)?, ctx.cwd())
}

pub fn run_script(
    exec: &crate::core::types::NativeScriptExecution,
    invocation_cwd: &Path,
) -> Result<std::process::ExitCode> {
    exec::run_script(exec, invocation_cwd)
}

pub fn run_local_bin(
    exec: &crate::core::types::NativeLocalBinExecution,
    cwd: &Path,
) -> Result<std::process::ExitCode> {
    exec::run_local_bin(exec, cwd)
}

pub fn format_debug(exec: &ResolvedExecution) -> String {
    exec::format_debug(exec)
}

fn into_attempt(decision: NativeDecision, cwd: &Path) -> Result<NativeAttempt> {
    Ok(match decision {
        NativeDecision::Eligible(plan) => NativeAttempt::Eligible(Box::new(match plan {
            NativePlan::Script(exec) => {
                ResolvedExecution::native_script(exec.script_name.clone(), cwd.to_path_buf(), exec)
            }
            NativePlan::LocalBin(exec) => {
                ResolvedExecution::native_local_bin(exec.bin_name.clone(), cwd.to_path_buf(), exec)
            }
        })),
        NativeDecision::Ineligible(reason) => NativeAttempt::Ineligible(reason.to_string()),
    })
}
