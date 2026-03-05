use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

mod support;

#[test]
fn ns_stops_on_first_failure() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let cwd = work.path();
        let marker = cwd.join("should-not-exist.txt");

        let fail_cmd = if cfg!(windows) { "exit /B 7" } else { "false" };
        let write_cmd = write_marker_command(&marker);

        let bin_dir = prepare_bin_dir(cwd);
        let output = run_alias_output(
            &bin_dir,
            "ns",
            vec!["-C", cwd.to_str().unwrap(), fail_cmd, &write_cmd],
            &[],
        );

        assert!(!output.status.success());
        assert!(!marker.exists());
    });
}

#[test]
fn np_waits_for_all_commands_then_fails() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let cwd = work.path();
        let marker = cwd.join("parallel-finished.txt");

        let fail_cmd = if cfg!(windows) { "exit /B 5" } else { "false" };
        let delayed_write = delayed_write_marker_command(&marker);

        let bin_dir = prepare_bin_dir(cwd);
        let output = run_alias_output(
            &bin_dir,
            "np",
            vec!["-C", cwd.to_str().unwrap(), fail_cmd, &delayed_write],
            &[],
        );

        assert!(!output.status.success());
        assert!(marker.exists());
    });
}

#[test]
fn debug_mode_does_not_execute_np_or_node_p() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let cwd = work.path();

        let marker_np = cwd.join("np-debug.txt");
        let cmd_np = write_marker_command(&marker_np);

        let marker_node = cwd.join("node-p-debug.txt");
        let cmd_node = write_marker_command(&marker_node);

        let bin_dir = prepare_bin_dir(cwd);

        let np_output = run_alias_output(
            &bin_dir,
            "np",
            vec!["-C", cwd.to_str().unwrap(), &cmd_np, "?"],
            &[],
        );
        assert!(np_output.status.success());
        let np_stdout = String::from_utf8_lossy(&np_output.stdout);
        assert!(np_stdout.contains("hni batch:parallel"));
        assert!(!marker_np.exists());

        let node_output = run_alias_output(
            &bin_dir,
            "node",
            vec!["-C", cwd.to_str().unwrap(), "p", &cmd_node, "?"],
            &[],
        );
        assert!(node_output.status.success());
        let node_stdout = String::from_utf8_lossy(&node_output.stdout);
        assert!(node_stdout.contains("hni batch:parallel"));
        assert!(!marker_node.exists());
    });
}

fn prepare_bin_dir(cwd: &Path) -> PathBuf {
    let bin_dir = cwd.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let exe = hni_executable_path();
    if !exe.exists() {
        panic!("hni executable not found at {}", exe.display());
    }

    create_alias(&exe, &bin_dir, "np");
    create_alias(&exe, &bin_dir, "ns");
    create_alias(&exe, &bin_dir, "node");
    bin_dir
}

fn hni_executable_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_hni") {
        return PathBuf::from(path);
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push(if cfg!(windows) { "hni.exe" } else { "hni" });
    path
}

fn run_alias_output(
    bin_dir: &Path,
    alias: &str,
    args: Vec<&str>,
    extra_env: &[(&str, &str)],
) -> Output {
    let alias_bin = if cfg!(windows) {
        format!("{alias}.exe")
    } else {
        alias.to_string()
    };

    let mut cmd = Command::new(bin_dir.join(alias_bin));
    cmd.args(args)
        .env("HNI_SKIP_PM_CHECK", "1")
        .env("HNI_AUTO_INSTALL", "false");

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    cmd.output().expect("failed to run alias binary")
}

fn write_marker_command(path: &Path) -> String {
    if cfg!(windows) {
        format!("echo ok > \"{}\"", path.display())
    } else {
        format!("echo ok > '{}'", path.display())
    }
}

fn delayed_write_marker_command(path: &Path) -> String {
    if cfg!(windows) {
        format!(
            "timeout /T 1 /NOBREAK >NUL & echo ok > \"{}\"",
            path.display()
        )
    } else {
        format!("sleep 0.2; echo ok > '{}'", path.display())
    }
}

fn create_alias(target: &Path, dir: &Path, alias: &str) {
    let alias_path = if cfg!(windows) {
        dir.join(format!("{alias}.exe"))
    } else {
        dir.join(alias)
    };

    if alias_path.exists() {
        fs::remove_file(&alias_path).unwrap();
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, alias_path).unwrap();
    }

    #[cfg(windows)]
    {
        fs::copy(target, alias_path).unwrap();
    }
}
