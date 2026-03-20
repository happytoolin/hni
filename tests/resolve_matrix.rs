use std::{fs, path::Path};

use hni::core::{
    config::{DefaultAgent, HniConfig, RunAgent},
    resolve::{self, ResolveContext},
    types::{ExecutionStrategy, NativeExecution, PackageManager},
};

mod support;

fn with_skip_pm_check<T>(f: impl FnOnce() -> T) -> T {
    support::with_env_lock(|| {
        struct SkipPmCheckGuard;
        impl Drop for SkipPmCheckGuard {
            fn drop(&mut self) {
                support::remove_var("HNI_SKIP_PM_CHECK");
            }
        }

        support::set_var("HNI_SKIP_PM_CHECK", "1");
        let guard = SkipPmCheckGuard;
        let out = f();
        drop(guard);
        out
    })
}

fn write_package_json(dir: &Path, raw: &str) {
    fs::write(dir.join("package.json"), raw).unwrap();
}

#[test]
fn ni_maps_npm_add_to_install() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["i", "vite"]);
    });
}

#[test]
fn ni_empty_args_installs_for_npm() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["i"]);
    });
}

#[test]
fn ni_only_flags_stays_install_mode_not_add_mode() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@1.22.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["--check-files".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["install", "--check-files"]);
    });
}

#[test]
fn ni_maps_yarn_add() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@1.22.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["add", "vite"]);
    });
}

#[test]
fn ni_bun_dev_flag_is_translated() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"bun@1.1.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["@types/node".into(), "-D".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "bun");
        assert_eq!(resolved.args, vec!["add", "@types/node", "-d"]);
    });
}

#[test]
fn ni_npm_prod_flag_is_translated_to_omit_dev() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["-P".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["i", "--omit=dev"]);
    });
}

#[test]
fn ni_non_npm_prod_flag_is_translated_to_production() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@1.22.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["-P".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["install", "--production"]);
    });
}

#[test]
fn ni_global_install_uses_configured_global_agent() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);

        let cfg = HniConfig {
            global_agent: PackageManager::Yarn,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_ni(vec!["-g".into(), "eslint".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["global", "add", "eslint"]);
    });
}

#[test]
fn ni_frozen_if_present_uses_clean_install_with_lock() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);
        fs::write(dir.path().join("package-lock.json"), "lock").unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["--frozen-if-present".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["ci"]);
    });
}

#[test]
fn ni_frozen_if_present_falls_back_to_install_without_lock() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["--frozen-if-present".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["i"]);
    });
}

#[test]
fn ni_frozen_always_uses_clean_install() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["--frozen".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["ci"]);
    });
}

#[test]
fn nr_maps_pnpm_run() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nr(vec!["dev".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["run", "dev"]);
    });
}

#[test]
fn nr_without_args_defaults_to_start() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nr(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["run", "start"]);
    });
}

#[test]
fn nr_npm_with_extra_args_inserts_double_dash() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nr(vec!["dev".into(), "--port=3000".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["run", "dev", "--", "--port=3000"]);
    });
}

#[test]
fn nr_if_present_for_npm_is_inserted_after_run() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved =
            resolve::resolve_nr(vec!["build".into(), "--if-present".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(resolved.args, vec!["run", "--if-present", "build"]);
    });
}

#[test]
fn nr_if_present_for_npm_with_extra_args_keeps_double_dash() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nr(
            vec!["dev".into(), "--if-present".into(), "--port=3000".into()],
            &ctx,
        )
        .unwrap();

        assert_eq!(resolved.program, "npm");
        assert_eq!(
            resolved.args,
            vec!["run", "--if-present", "dev", "--", "--port=3000"]
        );
    });
}

#[test]
fn nr_if_present_for_non_run_command_is_prefixed() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"deno@1.46.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved =
            resolve::resolve_nr(vec!["build".into(), "--if-present".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "deno");
        assert_eq!(resolved.args, vec!["task", "--if-present", "build"]);
    });
}

#[test]
fn nr_run_agent_node_uses_node_passthrough_run() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);

        let cfg = HniConfig {
            run_agent: RunAgent::Node,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nr(vec!["dev".into(), "--port=3000".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "node");
        assert_eq!(resolved.args, vec!["--run", "dev", "--port=3000"]);
        assert!(resolved.passthrough);
    });
}

#[test]
fn nr_native_mode_uses_native_script_execution() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(
            dir.path(),
            r#"{"packageManager":"npm@10.0.0","scripts":{"dev":"vite"}}"#,
        );

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nr(vec!["dev".into(), "--port=3000".into()], &ctx).unwrap();

        assert!(matches!(
            resolved.strategy,
            ExecutionStrategy::Native(NativeExecution::RunScript(_))
        ));
        assert_eq!(resolved.program, "dev");
        assert_eq!(resolved.args, vec!["--port=3000"]);
    });
}

#[test]
fn nr_native_mode_is_available_on_windows_too() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(
            dir.path(),
            r#"{"packageManager":"npm@10.0.0","scripts":{"dev":"echo ok"}}"#,
        );

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nr(vec!["dev".into()], &ctx).unwrap();

        assert!(matches!(
            resolved.strategy,
            ExecutionStrategy::Native(NativeExecution::RunScript(_))
        ));
    });
}

#[test]
fn nr_native_mode_if_present_missing_script_is_native_noop() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(
            dir.path(),
            r#"{"packageManager":"npm@10.0.0","scripts":{}}"#,
        );

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nr(
            vec!["missing".into(), "--if-present".into(), "--watch".into()],
            &ctx,
        )
        .unwrap();

        match &resolved.strategy {
            ExecutionStrategy::Native(NativeExecution::RunScript(exec)) => {
                assert!(exec.steps.is_empty());
                assert_eq!(exec.forwarded_args, vec!["--watch"]);
            }
            other => panic!("expected native script execution, got {other:?}"),
        }
    });
}

#[test]
fn nr_native_mode_falls_back_for_yarn_berry_pnp() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(
            dir.path(),
            r#"{"packageManager":"yarn@4.0.0","scripts":{"dev":"vite"}}"#,
        );
        fs::write(dir.path().join(".pnp.cjs"), "module.exports = {};\n").unwrap();

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nr(vec!["dev".into()], &ctx).unwrap();

        assert!(matches!(resolved.strategy, ExecutionStrategy::External));
        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["run", "dev"]);
        assert_eq!(
            resolved.native_fallback_reason.as_deref(),
            Some(
                "yarn berry Plug'n'Play does not expose node_modules/.bin; falling back to yarn execution"
            )
        );
    });
}

#[test]
fn nci_uses_immutable_for_yarn_berry() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@4.2.0"}"#);
        fs::write(dir.path().join("yarn.lock"), "# lock").unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nci(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["install", "--immutable"]);
    });
}

#[test]
fn nci_without_lockfile_falls_back_to_install() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nci(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["i"]);
    });
}

#[test]
fn nci_deno_lockfile_uses_frozen_install() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"deno@1.46.0"}"#);
        fs::write(dir.path().join("deno.lock"), "lock").unwrap();

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nci(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "deno");
        assert_eq!(resolved.args, vec!["install", "--frozen"]);
    });
}

#[test]
fn ni_in_workspace_package_uses_root_package_manager() {
    with_skip_pm_check(|| {
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("package.json"),
            r#"{"packageManager":"pnpm@9.0.0","workspaces":["packages/*"]}"#,
        )
        .unwrap();
        fs::write(root.path().join("pnpm-lock.yaml"), "lock").unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        write_package_json(&pkg, r#"{"name":"app"}"#);

        let ctx = ResolveContext::new(pkg, HniConfig::default());
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["add", "vite"]);
    });
}

#[test]
fn nci_in_workspace_package_uses_root_lockfile() {
    with_skip_pm_check(|| {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("pnpm-lock.yaml"), "lock").unwrap();

        let pkg = root.path().join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        write_package_json(&pkg, r#"{"name":"app"}"#);

        let ctx = ResolveContext::new(pkg, HniConfig::default());
        let resolved = resolve::resolve_nci(Vec::new(), &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["i", "--frozen-lockfile"]);
    });
}

#[test]
fn nlx_npm_uses_npx() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nlx(vec!["vitest".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "npx");
        assert_eq!(resolved.args, vec!["vitest"]);
    });
}

#[test]
fn nlx_native_mode_uses_local_bin_when_present() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);
        fs::create_dir_all(dir.path().join("node_modules").join(".bin")).unwrap();
        fs::write(
            dir.path().join("node_modules").join(".bin").join("vitest"),
            "",
        )
        .unwrap();

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nlx(vec!["vitest".into(), "--help".into()], &ctx).unwrap();

        match &resolved.strategy {
            ExecutionStrategy::Native(NativeExecution::RunLocalBin(exec)) => {
                assert_eq!(exec.bin_name, "vitest");
                assert!(exec.bin_path.ends_with("node_modules/.bin/vitest"));
            }
            other => panic!("expected native local bin execution, got {other:?}"),
        }
        assert_eq!(resolved.args, vec!["--help"]);
    });
}

#[test]
fn nlx_native_mode_falls_back_to_package_manager_when_local_bin_is_missing() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nlx(vec!["vitest".into()], &ctx).unwrap();

        assert!(matches!(resolved.strategy, ExecutionStrategy::External));
        assert_eq!(resolved.program, "npx");
        assert_eq!(
            resolved.native_fallback_reason.as_deref(),
            Some(
                "local binary not found in node_modules/.bin or package.json bin entries; falling back to package-manager exec"
            )
        );
    });
}

#[test]
fn nlx_native_mode_uses_declared_package_bin_when_present() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bin")).unwrap();
        fs::write(dir.path().join("bin").join("hello.js"), "console.log('hi')").unwrap();
        write_package_json(
            dir.path(),
            r#"{"name":"tooling","packageManager":"npm@10.0.0","bin":{"hello":"bin/hello.js"}}"#,
        );

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nlx(vec!["hello".into(), "--flag".into()], &ctx).unwrap();

        match &resolved.strategy {
            ExecutionStrategy::Native(NativeExecution::RunLocalBin(exec)) => {
                assert_eq!(exec.bin_name, "hello");
                assert!(exec.bin_path.ends_with("bin/hello.js"));
            }
            other => panic!("expected native local bin execution, got {other:?}"),
        }
        assert_eq!(resolved.args, vec!["--flag"]);
    });
}

#[test]
fn nlx_native_mode_uses_pnpm_hoisted_bin_dir_when_present() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);
        let bin_dir = dir
            .path()
            .join("node_modules")
            .join(".pnpm")
            .join("node_modules")
            .join(".bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(
            bin_dir.join(if cfg!(windows) {
                "vitest.cmd"
            } else {
                "vitest"
            }),
            "",
        )
        .unwrap();

        let cfg = HniConfig {
            native_mode: true,
            ..HniConfig::default()
        };
        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nlx(vec!["vitest".into()], &ctx).unwrap();

        match &resolved.strategy {
            ExecutionStrategy::Native(NativeExecution::RunLocalBin(exec)) => {
                assert!(
                    exec.bin_path
                        .to_string_lossy()
                        .contains("node_modules/.pnpm/node_modules/.bin")
                );
            }
            other => panic!("expected native local bin execution, got {other:?}"),
        }
    });
}

#[test]
fn nlx_pnpm_uses_dlx() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nlx(vec!["vitest".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["dlx", "vitest"]);
    });
}

#[test]
fn nlx_yarn_berry_uses_dlx() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@4.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nlx(vec!["vitest".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["dlx", "vitest"]);
    });
}

#[test]
fn nlx_bun_uses_x() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"bun@1.1.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nlx(vec!["vitest".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "bun");
        assert_eq!(resolved.args, vec!["x", "vitest"]);
    });
}

#[test]
fn nlx_deno_wraps_target_with_npm_prefix() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"deno@1.46.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nlx(vec!["vitest".into(), "--help".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "deno");
        assert_eq!(resolved.args, vec!["run", "npm:vitest", "--help"]);
    });
}

#[test]
fn nu_interactive_for_yarn_classic_uses_upgrade_interactive() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@1.22.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nu(vec!["-i".into(), "vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["upgrade-interactive", "vite"]);
    });
}

#[test]
fn nu_interactive_for_yarn_berry_uses_up_i() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"yarn@4.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved =
            resolve::resolve_nu(vec!["--interactive".into(), "vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["up", "-i", "vite"]);
    });
}

#[test]
fn nu_interactive_for_deno_maps_to_outdated_update() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"deno@1.46.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved = resolve::resolve_nu(vec!["-i".into(), "jsr:@std/fs".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "deno");
        assert_eq!(resolved.args, vec!["outdated", "--update", "jsr:@std/fs"]);
    });
}

#[test]
fn nu_interactive_rejected_for_npm() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let err = resolve::resolve_nu(vec!["-i".into()], &ctx).unwrap_err();

        assert!(
            err.to_string()
                .contains("interactive upgrade is not supported for npm")
        );
    });
}

#[test]
fn nu_interactive_for_pnpm_uses_update_i() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"pnpm@9.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let resolved =
            resolve::resolve_nu(vec!["--interactive".into(), "vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "pnpm");
        assert_eq!(resolved.args, vec!["update", "-i", "vite"]);
    });
}

#[test]
fn nun_global_without_target_errors_after_stripping_g_flag() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let err = resolve::resolve_nun(vec!["-g".into()], &ctx).unwrap_err();

        assert!(
            err.to_string()
                .contains("no dependencies selected for uninstall")
        );
    });
}

#[test]
fn nun_requires_at_least_one_dependency() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let ctx = ResolveContext::new(dir.path().to_path_buf(), HniConfig::default());
        let err = resolve::resolve_nun(Vec::new(), &ctx).unwrap_err();

        assert!(
            err.to_string()
                .contains("no dependencies selected for uninstall")
        );
    });
}

#[test]
fn nun_global_uses_configured_global_agent() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let cfg = HniConfig {
            global_agent: PackageManager::Yarn,
            ..HniConfig::default()
        };

        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_nun(vec!["-g".into(), "eslint".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "yarn");
        assert_eq!(resolved.args, vec!["global", "remove", "eslint"]);
    });
}

#[test]
fn ni_global_rejects_yarn_berry_agent() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let cfg = HniConfig {
            global_agent: PackageManager::YarnBerry,
            ..HniConfig::default()
        };

        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let err = resolve::resolve_ni(vec!["-g".into(), "eslint".into()], &ctx).unwrap_err();

        assert!(
            err.to_string()
                .contains("global install/uninstall is not supported by yarn (berry)")
        );
    });
}

#[test]
fn nun_global_rejects_yarn_berry_agent() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        write_package_json(dir.path(), r#"{"packageManager":"npm@10.0.0"}"#);

        let cfg = HniConfig {
            global_agent: PackageManager::YarnBerry,
            ..HniConfig::default()
        };

        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let err = resolve::resolve_nun(vec!["-g".into(), "eslint".into()], &ctx).unwrap_err();

        assert!(
            err.to_string()
                .contains("global install/uninstall is not supported by yarn (berry)")
        );
    });
}

#[test]
fn resolves_from_default_agent_when_no_project_hints_exist() {
    with_skip_pm_check(|| {
        let dir = tempfile::tempdir().unwrap();
        let cfg = HniConfig {
            default_agent: DefaultAgent::Agent(PackageManager::Bun),
            ..HniConfig::default()
        };

        let ctx = ResolveContext::new(dir.path().to_path_buf(), cfg);
        let resolved = resolve::resolve_ni(vec!["vite".into()], &ctx).unwrap();

        assert_eq!(resolved.program, "bun");
        assert_eq!(resolved.args, vec!["add", "vite"]);
    });
}
