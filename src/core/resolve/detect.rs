use crate::core::{
    detect::ensure_package_manager_available,
    error::{HniError, HniResult},
    types::{DetectionResult, DetectionSource, PackageManager},
};

use super::context::ResolveContext;

#[derive(Debug, Clone)]
pub(super) struct AgentResolution {
    pub pm: PackageManager,
    pub has_lock: bool,
    pub version_hint: Option<String>,
}

pub fn detected_package_manager(ctx: &ResolveContext) -> HniResult<PackageManager> {
    let detected = detect_for_action(ctx, false)?;
    Ok(detected.pm)
}

pub(super) fn detect_for_action(
    ctx: &ResolveContext,
    use_global: bool,
) -> HniResult<AgentResolution> {
    let cwd = ctx.cwd();
    let config = &ctx.config;
    let detection = if use_global {
        DetectionResult {
            agent: Some(config.global_agent),
            has_lock: false,
            version_hint: None,
            source: DetectionSource::Config,
        }
    } else {
        ctx.detect()
            .map_err(|error| HniError::detection(error.to_string()))?
    };

    let pm = detection.agent.ok_or_else(|| {
        HniError::detection(format!(
            "unable to detect package manager in {}.\nAdd packageManager to package.json, add a lockfile, or set defaultAgent in ~/.hnirc",
            cwd.display()
        ))
    })?;

    if use_global && pm == PackageManager::YarnBerry {
        return Err(HniError::detection(
            "global install/uninstall is not supported by yarn (berry).\nUse a different globalAgent (for example: npm, pnpm, yarn, bun, deno).",
        ));
    }

    Ok(AgentResolution {
        pm,
        has_lock: detection.has_lock,
        version_hint: detection.version_hint,
    })
}

pub(super) fn ensure_detected_available(
    resolution: &AgentResolution,
    ctx: &ResolveContext,
) -> HniResult<()> {
    if !ctx.should_verify_package_manager_availability() {
        return Ok(());
    }

    ensure_package_manager_available(
        resolution.pm,
        resolution.version_hint.as_deref(),
        &ctx.config,
        ctx.cwd(),
    )
    .map_err(|error| HniError::detection(error.to_string()))
}
