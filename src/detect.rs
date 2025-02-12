use which::which;
use std::path::Path;

pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    pub fn detect(cwd: &Path) -> Option<Self> {
        if cwd.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        }
        if cwd.join("yarn.lock").exists() {
            return Some(Self::Yarn);
        }
        if which("npm").is_ok() {
            return Some(Self::Npm);
        }
        None
    }
}
