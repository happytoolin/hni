use std::path::Path;

use crate::core::{
    config::HniConfig,
    detect::{detect, ensure_package_manager_available},
    error::{HniError, HniResult},
    types::{DetectionResult, DetectionSource, PackageManager},
};

use super::context::ResolveContext;

#[derive(Debug, Clone, Copy)]
pub(super) struct AgentResolution {
    pub pm: PackageManager,
    pub has_lock: bool,
}

pub fn detected_package_manager(ctx: &ResolveContext) -> HniResult<PackageManager> {
    let detected = detect_for_action(&ctx.cwd, &ctx.config, false)?;
    Ok(detected.pm)
}

pub(super) fn detect_for_action(
    cwd: &Path,
    config: &HniConfig,
    use_global: bool,
) -> HniResult<AgentResolution> {
    let detection = if use_global {
        DetectionResult {
            agent: Some(config.global_agent),
            has_lock: false,
            version_hint: None,
            source: DetectionSource::Config,
        }
    } else {
        detect(cwd, config).map_err(|error| HniError::detection(error.to_string()))?
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

    ensure_package_manager_available(pm, detection.version_hint.as_deref(), config, cwd)
        .map_err(|error| HniError::detection(error.to_string()))?;

    Ok(AgentResolution {
        pm,
        has_lock: detection.has_lock,
    })
}
