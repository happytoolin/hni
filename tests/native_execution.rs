use std::fs;

mod support;

use support::run_hni;

#[test]
fn native_nr_runs_hooks_from_nearest_package_and_forwards_args() {
    if cfg!(windows) {
        return;
    }

    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let root = work.path().join("workspace");
        let pkg = root.join("packages").join("app");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(root.join("package-lock.json"), "lock").unwrap();
        fs::write(root.join("package.json"), r#"{"name":"workspace"}"#).unwrap();
        fs::write(
            pkg.join("package.json"),
            r#"{"name":"app","scripts":{"predev":"printf 'pre' >> order.txt","dev":"printf '%s' \"$*\" > args.txt; printf 'dev' >> order.txt","postdev":"printf 'post' >> order.txt"}}"#,
        )
        .unwrap();

        let output = run_hni(
            vec![
                "nr",
                "-C",
                pkg.to_str().unwrap(),
                "--native",
                "dev",
                "--",
                "alpha",
                "beta",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );

        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            fs::read_to_string(pkg.join("order.txt")).unwrap(),
            "predevpost"
        );
        assert_eq!(
            fs::read_to_string(pkg.join("args.txt")).unwrap(),
            "alpha beta"
        );
    });
}

#[test]
fn native_nlx_runs_local_bin_directly() {
    if cfg!(windows) {
        return;
    }

    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("project");
        let bin_dir = project.join("node_modules").join(".bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(project.join("package.json"), r#"{"name":"x"}"#).unwrap();

        let bin = bin_dir.join("hello");
        fs::write(&bin, "#!/bin/sh\nprintf '%s' \"$*\" > bin-args.txt\n").unwrap();
        make_executable(&bin);

        let output = run_hni(
            vec![
                "nlx",
                "-C",
                project.to_str().unwrap(),
                "--native",
                "hello",
                "world",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );

        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            fs::read_to_string(project.join("bin-args.txt")).unwrap(),
            "world"
        );
    });
}

#[test]
fn native_explain_reports_fallback_reason() {
    if cfg!(windows) {
        return;
    }

    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"name":"x","scripts":{"dev":"echo $npm_package_name"}}"#,
        )
        .unwrap();

        let output = run_hni(
            vec![
                "nr",
                "-C",
                project.to_str().unwrap(),
                "--native",
                "--explain",
                "dev",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );

        assert!(output.status.success(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("native_mode: true"));
        assert!(stdout.contains("execution_mode: delegated"));
        assert!(stdout.contains("native_status: fallback"));
        assert!(stdout.contains("native_fallback_reason:"));
        assert!(stdout.contains("resolved: npm run dev"));
    });
}

#[test]
fn node_run_and_exec_inherit_native_resolution() {
    if cfg!(windows) {
        return;
    }

    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("project");
        let bin_dir = project.join("node_modules").join(".bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"name":"x","scripts":{"dev":"echo ok"}}"#,
        )
        .unwrap();

        let bin = bin_dir.join("hello");
        fs::write(&bin, "#!/bin/sh\nexit 0\n").unwrap();
        make_executable(&bin);

        let run_output = run_hni(
            vec![
                "node",
                "-C",
                project.to_str().unwrap(),
                "--native",
                "--debug-resolved",
                "run",
                "dev",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(run_output.status.success(), "{run_output:?}");
        assert_eq!(
            String::from_utf8_lossy(&run_output.stdout).trim(),
            "hni native:run-script dev"
        );

        let exec_output = run_hni(
            vec![
                "node",
                "-C",
                project.to_str().unwrap(),
                "--native",
                "--debug-resolved",
                "exec",
                "hello",
                "world",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(exec_output.status.success(), "{exec_output:?}");
        assert_eq!(
            String::from_utf8_lossy(&exec_output.stdout).trim(),
            "hni native:run-local-bin hello world"
        );
    });
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}
