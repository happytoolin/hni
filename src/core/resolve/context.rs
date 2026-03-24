use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::core::{
    config::{DefaultAgent, HniConfig},
    detect::{detect_lockfile_in_dir, parse_package_manager_field},
    error::HniResult,
    package::NearestPackage,
    pkg_json::{PackageJson, package_json_path, read_package_json},
    types::{DetectionResult, DetectionSource, PackageManager},
};

#[derive(Debug)]
pub struct ResolveContext {
    pub cwd: PathBuf,
    pub config: HniConfig,
    project_state: OnceLock<ProjectState>,
}

impl ResolveContext {
    pub fn new(cwd: PathBuf, config: HniConfig) -> Self {
        Self {
            cwd,
            config,
            project_state: OnceLock::new(),
        }
    }

    pub(crate) fn project_state(&self) -> HniResult<&ProjectState> {
        if let Some(state) = self.project_state.get() {
            return Ok(state);
        }

        let state = ProjectState::scan(&self.cwd)?;
        let _ = self.project_state.set(state);
        Ok(self
            .project_state
            .get()
            .expect("project state should be initialized"))
    }

    pub fn detect(&self) -> HniResult<DetectionResult> {
        Ok(self.project_state()?.detect(&self.config))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectState {
    ancestors: Vec<AncestorState>,
    nearest_package: Option<NearestPackage>,
    bin_dirs: Vec<PathBuf>,
    has_yarn_pnp_loader: bool,
}

#[derive(Debug, Clone)]
struct AncestorState {
    dir: PathBuf,
    manifest: Option<PackageJson>,
    lockfile_pm: Option<PackageManager>,
}

impl ProjectState {
    fn scan(cwd: &Path) -> HniResult<Self> {
        let mut ancestors = Vec::new();
        let mut nearest_package = None;
        let mut bin_dirs = Vec::new();
        let mut has_yarn_pnp_loader = false;

        for dir in cwd.ancestors() {
            let dir = dir.to_path_buf();
            let manifest = read_package_json(&dir)?;
            let package_json_path = package_json_path(&dir);
            let lockfile_pm = detect_lockfile_in_dir(&dir);

            if nearest_package.is_none()
                && let Some(manifest) = manifest.clone()
            {
                nearest_package = Some(NearestPackage {
                    root: dir.clone(),
                    package_json_path: package_json_path.clone(),
                    manifest,
                });
            }

            for candidate in [
                dir.join("node_modules").join(".bin"),
                dir.join("node_modules")
                    .join(".pnpm")
                    .join("node_modules")
                    .join(".bin"),
            ] {
                if candidate.is_dir() {
                    bin_dirs.push(candidate);
                }
            }

            has_yarn_pnp_loader |= dir.join(".pnp.cjs").exists() || dir.join(".pnp.js").exists();

            ancestors.push(AncestorState {
                dir,
                manifest,
                lockfile_pm,
            });
        }

        Ok(Self {
            ancestors,
            nearest_package,
            bin_dirs,
            has_yarn_pnp_loader,
        })
    }

    pub(crate) fn nearest_package(&self) -> Option<NearestPackage> {
        self.nearest_package.clone()
    }

    pub(crate) fn bin_dirs(&self) -> &[PathBuf] {
        &self.bin_dirs
    }

    pub(crate) fn has_yarn_pnp_loader(&self) -> bool {
        self.has_yarn_pnp_loader
    }

    pub(crate) fn resolve_declared_package_bin(&self, bin_name: &str) -> Option<PathBuf> {
        self.ancestors.iter().find_map(|ancestor| {
            let manifest = ancestor.manifest.as_ref()?;
            let relative = manifest.bin_command_path(bin_name)?;
            let candidate = ancestor.dir.join(relative);
            candidate.is_file().then_some(candidate)
        })
    }

    fn detect(&self, config: &HniConfig) -> DetectionResult {
        let mut has_lock = false;
        let mut resolved = None;

        for ancestor in &self.ancestors {
            has_lock |= ancestor.lockfile_pm.is_some();

            if resolved.is_none() {
                let package_manager_hint = ancestor
                    .manifest
                    .as_ref()
                    .and_then(|package_json| package_json.package_manager.as_deref())
                    .and_then(parse_package_manager_field);

                if let Some((pm, version_hint)) = package_manager_hint {
                    resolved = Some(DetectionResult {
                        agent: Some(pm),
                        has_lock,
                        version_hint,
                        source: DetectionSource::PackageManagerField,
                    });
                } else if let Some(pm) = ancestor.lockfile_pm {
                    resolved = Some(DetectionResult {
                        agent: Some(pm),
                        has_lock,
                        version_hint: None,
                        source: DetectionSource::Lockfile,
                    });
                }
            }

            if resolved.is_some() && has_lock {
                break;
            }
        }

        if let Some(mut resolved) = resolved {
            resolved.has_lock = has_lock;
            return resolved;
        }

        if let DefaultAgent::Agent(agent) = config.default_agent {
            return DetectionResult {
                agent: Some(agent),
                has_lock,
                version_hint: None,
                source: DetectionSource::Config,
            };
        }

        if which::which("npm").is_ok() {
            return DetectionResult {
                agent: Some(PackageManager::Npm),
                has_lock,
                version_hint: None,
                source: DetectionSource::Fallback,
            };
        }

        DetectionResult {
            agent: None,
            has_lock,
            version_hint: None,
            source: DetectionSource::None,
        }
    }
}
