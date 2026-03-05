use std::fs;

use hni::core::{
    config::HniConfig,
    resolve::{self, ResolveContext},
};

mod support;

#[test]
fn ni_maps_npm_add_to_install() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"packageManager":"npm@10.0.0"}"#,
        )
        .unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["i", "vite"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}

#[test]
fn ni_maps_yarn_add() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"packageManager":"yarn@1.22.0"}"#,
        )
        .unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["add", "vite"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}

#[test]
fn ni_bun_dev_flag_is_translated() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"packageManager":"bun@1.1.0"}"#,
        )
        .unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["@types/node".into(), "-D".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "bun");
        assert_eq!(resolved.args, vec!["add", "@types/node", "-d"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}

#[test]
fn nr_maps_pnpm_run() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"packageManager":"pnpm@9.0.0"}"#,
        )
        .unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nr(vec!["dev".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["run", "dev"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}

#[test]
fn nci_uses_immutable_for_yarn_berry() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"packageManager":"yarn@4.2.0"}"#,
        )
        .unwrap();
        fs::write(dir.path().join("yarn.lock"), "# lock").unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nci(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["install", "--immutable"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}

#[test]
fn ni_in_workspace_package_uses_root_package_manager() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");

        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("package.json"),
            r#"{"packageManager":"pnpm@9.0.0","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(root.path().join("pnpm-lock.yaml"), "lock").unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("package.json"), r#"{"name":"app"}"#).unwrap();

        let ctx = ResolveContext::new(pkg, HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["add", "vite"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}

#[test]
fn nci_in_workspace_package_uses_root_lockfile() {
    support::with_env_lock(|| {
        std::env::set_var("HNI_SKIP_PM_CHECK", "1");

        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("pnpm-lock.yaml"), "lock").unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("package.json"), r#"{"name":"app"}"#).unwrap();

        let ctx = ResolveContext::new(pkg, HniConfig::default());
        let resolved = resolve::resolve_nci(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["i", "--frozen-lockfile"]);
        std::env::remove_var("HNI_SKIP_PM_CHECK");
    });
}
