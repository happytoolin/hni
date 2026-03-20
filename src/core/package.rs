use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use super::{
    error::HniResult,
    pkg_json::{PackageJson, package_json_path, read_package_json},
};

#[derive(Debug, Clone)]
pub struct NearestPackage {
    pub root: PathBuf,
    pub package_json_path: PathBuf,
    pub manifest: PackageJson,
}

pub fn find_nearest_package(cwd: &Path) -> HniResult<Option<NearestPackage>> {
    for dir in cwd.ancestors() {
        if let Some(manifest) = read_package_json(dir)? {
            return Ok(Some(NearestPackage {
                root: dir.to_path_buf(),
                package_json_path: package_json_path(dir),
                manifest,
            }));
        }
    }

    Ok(None)
}

pub fn node_modules_bin_dirs(cwd: &Path) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut dirs = Vec::new();

    for dir in cwd.ancestors() {
        for candidate in [
            dir.join("node_modules").join(".bin"),
            dir.join("node_modules")
                .join(".pnpm")
                .join("node_modules")
                .join(".bin"),
        ] {
            if candidate.exists() && seen.insert(candidate.clone()) {
                dirs.push(candidate);
            }
        }
    }

    dirs
}

pub fn resolve_local_bin(bin_name: &str, bin_dirs: &[PathBuf]) -> Option<PathBuf> {
    #[cfg(windows)]
    const SUFFIXES: &[&str] = &["", ".cmd", ".exe", ".bat", ".ps1"];
    #[cfg(not(windows))]
    const SUFFIXES: &[&str] = &[""];

    for dir in bin_dirs {
        for suffix in SUFFIXES {
            let candidate = dir.join(format!("{bin_name}{suffix}"));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

pub fn resolve_declared_package_bin(cwd: &Path, bin_name: &str) -> HniResult<Option<PathBuf>> {
    for dir in cwd.ancestors() {
        let Some(manifest) = read_package_json(dir)? else {
            continue;
        };
        let Some(relative) = manifest.bin_command_path(bin_name) else {
            continue;
        };
        let candidate = dir.join(relative);
        if candidate.exists() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_nearest_package_in_ancestors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let nested = root.join("packages").join("app");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("package.json"), r#"{"name":"root"}"#).unwrap();

        let found = find_nearest_package(&nested).unwrap().unwrap();
        assert_eq!(found.root, root);
    }

    #[test]
    fn bin_dirs_are_nearest_first() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let nested = root.join("packages").join("app");
        fs::create_dir_all(root.join("node_modules").join(".bin")).unwrap();
        fs::create_dir_all(nested.join("node_modules").join(".bin")).unwrap();

        let bins = node_modules_bin_dirs(&nested);
        assert_eq!(bins[0], nested.join("node_modules").join(".bin"));
        assert_eq!(bins[1], root.join("node_modules").join(".bin"));
    }

    #[test]
    fn bin_dirs_include_pnpm_hoisted_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(
            root.join("node_modules")
                .join(".pnpm")
                .join("node_modules")
                .join(".bin"),
        )
        .unwrap();

        let bins = node_modules_bin_dirs(&root);
        assert_eq!(
            bins,
            vec![
                root.join("node_modules")
                    .join(".pnpm")
                    .join("node_modules")
                    .join(".bin")
            ]
        );
    }

    #[test]
    fn resolves_declared_package_bin_from_nearest_package() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg");
        let nested = pkg.join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir_all(pkg.join("bin")).unwrap();
        fs::write(
            pkg.join("package.json"),
            r#"{"name":"tooling","bin":{"hello":"bin/hello.js"}}"#,
        )
        .unwrap();
        fs::write(pkg.join("bin").join("hello.js"), "console.log('hi')").unwrap();

        let resolved = resolve_declared_package_bin(&nested, "hello").unwrap();
        assert_eq!(resolved, Some(pkg.join("bin").join("hello.js")));
    }
}
