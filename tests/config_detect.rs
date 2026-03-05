use std::fs;

use hni::core::{
    config::{DefaultAgent, HniConfig, RunAgent},
    detect::detect,
    types::{DetectionSource, PackageManager},
};

mod support;

#[test]
fn config_loads_and_env_overrides() {
    support::with_env_lock(|| {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("nirc");
        fs::write(
            &cfg_path,
            "defaultAgent=pnpm\nglobalAgent=yarn\nrunAgent=node\nuseSfw=true\n",
        )
        .unwrap();

        std::env::set_var("HNI_CONFIG_FILE", &cfg_path);
        std::env::set_var("HNI_GLOBAL_AGENT", "npm");
        std::env::set_var("HNI_AUTO_INSTALL", "true");

        let cfg = HniConfig::load().unwrap();
        assert_eq!(cfg.default_agent, DefaultAgent::Agent(PackageManager::Pnpm));
        assert_eq!(cfg.global_agent, PackageManager::Npm);
        assert_eq!(cfg.run_agent, RunAgent::Node);
        assert!(cfg.use_sfw);
        assert!(cfg.auto_install);

        std::env::remove_var("HNI_CONFIG_FILE");
        std::env::remove_var("HNI_GLOBAL_AGENT");
        std::env::remove_var("HNI_AUTO_INSTALL");
    });
}

#[test]
fn detect_prefers_package_manager_field_over_lockfile() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("pnpm-lock.yaml"), "lockfileVersion: '9.0'").unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"packageManager":"yarn@4.0.0"}"#,
    )
    .unwrap();

    let cfg = HniConfig::default();
    let detected = detect(dir.path(), &cfg).unwrap();

    assert_eq!(detected.agent, Some(PackageManager::YarnBerry));
    assert_eq!(detected.source, DetectionSource::PackageManagerField);
}

#[test]
fn detect_uses_config_fallback_when_no_lock_or_package_manager() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = HniConfig {
        default_agent: DefaultAgent::Agent(PackageManager::Bun),
        ..HniConfig::default()
    };

    let detected = detect(dir.path(), &cfg).unwrap();
    assert_eq!(detected.agent, Some(PackageManager::Bun));
    assert_eq!(detected.source, DetectionSource::Config);
}
