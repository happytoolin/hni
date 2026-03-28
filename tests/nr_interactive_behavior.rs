use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

mod support;

#[test]
fn repeat_last_and_dash_alias_reuse_previous_command() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("proj");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"name":"x","scripts":{"dev":"vite","build":"vite build"}}"#,
        )
        .unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();

        let bin_dir = prepare_nr_alias_dir(work.path());
        let storage_home = work.path().join("storage-home");
        fs::create_dir_all(&storage_home).unwrap();

        let envs = storage_envs(&storage_home);

        let first = run_nr(
            &bin_dir,
            vec!["-C", project.to_str().unwrap(), "--pm", "dev", "?"],
            &envs,
        );
        assert!(first.status.success());
        assert_eq!(String::from_utf8_lossy(&first.stdout).trim(), "npm run dev");

        let repeat_flag = run_nr(
            &bin_dir,
            vec![
                "-C",
                project.to_str().unwrap(),
                "--pm",
                "--repeat-last",
                "?",
            ],
            &envs,
        );
        assert!(repeat_flag.status.success());
        assert_eq!(
            String::from_utf8_lossy(&repeat_flag.stdout).trim(),
            "npm run dev"
        );

        let repeat_dash = run_nr(
            &bin_dir,
            vec!["-C", project.to_str().unwrap(), "--pm", "-", "?"],
            &envs,
        );
        assert!(repeat_dash.status.success());
        assert_eq!(
            String::from_utf8_lossy(&repeat_dash.stdout).trim(),
            "npm run dev"
        );
    });
}

#[test]
fn repeat_last_errors_when_no_previous_command_exists() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("proj");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("package.json"), r#"{"name":"x"}"#).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();

        let bin_dir = prepare_nr_alias_dir(work.path());
        let storage_home = work.path().join("storage-home-empty");
        fs::create_dir_all(&storage_home).unwrap();

        let envs = storage_envs(&storage_home);
        let out = run_nr(
            &bin_dir,
            vec!["-C", project.to_str().unwrap(), "--repeat-last"],
            &envs,
        );

        assert!(!out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("no last command found"));
    });
}

#[test]
fn completion_query_filters_scripts_by_prefix() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("proj");
        fs::create_dir_all(&project).unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"name":"x","scripts":{"dev":"vite","build":"vite build","test":"vitest"}}"#,
        )
        .unwrap();

        let bin_dir = prepare_nr_alias_dir(work.path());
        let storage_home = work.path().join("storage-home-completion");
        fs::create_dir_all(&storage_home).unwrap();

        let mut envs = storage_envs(&storage_home);
        envs.push(("COMP_CWORD".to_string(), "2".to_string()));

        let out = run_nr(
            &bin_dir,
            vec!["-C", project.to_str().unwrap(), "--completion", "d"],
            &envs,
        );
        assert!(out.status.success());

        let stdout = String::from_utf8_lossy(&out.stdout);
        let lines = stdout.lines().collect::<Vec<_>>();
        assert_eq!(lines, vec!["dev"]);
    });
}

fn prepare_nr_alias_dir(workdir: &Path) -> PathBuf {
    let dir = workdir.join("bin");
    fs::create_dir_all(&dir).unwrap();

    let exe = hni_executable_path();
    if !exe.exists() {
        panic!("hni executable not found at {}", exe.display());
    }

    create_alias(&exe, &dir, "nr");
    dir
}

fn storage_envs(base: &Path) -> Vec<(String, String)> {
    vec![
        (
            "XDG_CONFIG_HOME".to_string(),
            base.to_string_lossy().to_string(),
        ),
        ("HOME".to_string(), base.to_string_lossy().to_string()),
        ("APPDATA".to_string(), base.to_string_lossy().to_string()),
    ]
}

fn run_nr(bin_dir: &Path, args: Vec<&str>, extra_env: &[(String, String)]) -> Output {
    let alias = if cfg!(windows) { "nr.exe" } else { "nr" };

    let mut cmd = Command::new(bin_dir.join(alias));
    cmd.args(args)
        .env("HNI_SKIP_PM_CHECK", "1")
        .env("HNI_AUTO_INSTALL", "false");

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    cmd.output().expect("failed to run nr alias")
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
