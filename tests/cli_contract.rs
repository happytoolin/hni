use std::fs;

mod support;

use support::run_hni;

#[test]
fn help_and_version_contracts_are_hni_first() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("npm");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(project.join("package.json"), r#"{"name":"x"}"#).unwrap();

        let help_subcommand = run_hni(vec!["help", "ni"], &[("HNI_SKIP_PM_CHECK", "1")]);
        assert!(help_subcommand.status.success());
        let help_subcommand_out = String::from_utf8_lossy(&help_subcommand.stdout);
        assert!(help_subcommand_out.contains("Usage: ni"));

        let help_flag = run_hni(
            vec!["ni", "-C", project.to_str().unwrap(), "--help"],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(help_flag.status.success());
        let help_flag_out = String::from_utf8_lossy(&help_flag.stdout);
        assert!(help_flag_out.contains("Usage: ni"));
        assert!(!help_flag_out.contains("Usage:\nnpm install"));

        let passthrough_help = run_hni(
            vec![
                "ni",
                "-C",
                project.to_str().unwrap(),
                "--debug-resolved",
                "--",
                "--help",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(passthrough_help.status.success());
        let passthrough_help_out = String::from_utf8_lossy(&passthrough_help.stdout);
        assert_eq!(passthrough_help_out.trim(), "npm i --help");

        let version = run_hni(
            vec!["ni", "-C", project.to_str().unwrap(), "--version"],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(version.status.success());
        let version_out = String::from_utf8_lossy(&version.stdout);
        assert!(version_out.contains("hni       v"));
    });
}

#[test]
fn global_flags_work_anywhere_before_passthrough_separator() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("npm");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(project.join("package.json"), r#"{"name":"x"}"#).unwrap();

        let output = run_hni(
            vec![
                "ni",
                "-C",
                project.to_str().unwrap(),
                "vite",
                "--debug-resolved",
            ],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "npm i vite");
    });
}

#[test]
fn deprecated_question_mark_debug_alias_still_works_with_warning() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("npm");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(project.join("package.json"), r#"{"name":"x"}"#).unwrap();

        let output = run_hni(
            vec!["ni", "-C", project.to_str().unwrap(), "vite", "?"],
            &[("HNI_SKIP_PM_CHECK", "1")],
        );
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "npm i vite");
        assert!(String::from_utf8_lossy(&output.stderr).contains("deprecated"));
    });
}

#[test]
fn native_cli_flags_override_environment_setting() {
    support::with_env_lock(|| {
        let work = tempfile::tempdir().unwrap();
        let project = work.path().join("npm");
        fs::create_dir_all(project.join("node_modules").join(".bin")).unwrap();
        fs::write(project.join("package-lock.json"), "lock").unwrap();
        fs::write(
            project.join("package.json"),
            r#"{"name":"x","scripts":{"dev":"vite"}}"#,
        )
        .unwrap();
        fs::write(project.join("node_modules").join(".bin").join("vite"), "").unwrap();

        let force_native = run_hni(
            vec![
                "nr",
                "-C",
                project.to_str().unwrap(),
                "--native",
                "--debug-resolved",
                "dev",
            ],
            &[("HNI_SKIP_PM_CHECK", "1"), ("HNI_NATIVE", "false")],
        );
        assert!(force_native.status.success());
        assert_eq!(
            String::from_utf8_lossy(&force_native.stdout).trim(),
            "hni native:run-script dev"
        );

        let disable_native = run_hni(
            vec![
                "nr",
                "-C",
                project.to_str().unwrap(),
                "--no-native",
                "--debug-resolved",
                "dev",
            ],
            &[("HNI_SKIP_PM_CHECK", "1"), ("HNI_NATIVE", "true")],
        );
        assert!(disable_native.status.success());
        assert_eq!(
            String::from_utf8_lossy(&disable_native.stdout).trim(),
            "npm run dev"
        );
    });
}
