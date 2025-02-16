use std::env;
use std::fs;
use std::fs::File;

use anyhow::Result;
use tempfile::tempdir;

use nirs::detect::{detect, detect_sync, PackageManagerFactoryEnum};

mod common;

use common::create_empty_file;

#[test]
fn test_detect_package_manager_from_package_json() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(temp_dir.path().join("package.json").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
        "packageManager": "yarn@1.22.0"
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Yarn));
    Ok(())
}

#[test]
fn test_detect_package_manager_from_lockfile() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(temp_dir.path().join("yarn.lock").to_str().unwrap())?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Yarn));
    Ok(())
}

#[test]
fn test_detect_invalid_directory() -> Result<()> {
    let temp_dir = tempdir()?;
    let invalid_path = temp_dir.path().join("invalid");
    let result = detect(&invalid_path)?;
    assert_eq!(result, None);
    Ok(())
}

#[test]
fn test_detect_sync_no_package_manager() {
    let dir = tempdir().unwrap();
    env::set_var("PATH", "");
    assert_eq!(detect_sync(dir.path()), None);
}

#[test]
fn test_detect_sync_package_manager_from_package_json() {
    let temp_dir = tempdir().unwrap();
    create_empty_file(temp_dir.path().join("package.json").to_str().unwrap()).unwrap();
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
        "packageManager": "npm@8.0.0"
    }"#,
    )
    .unwrap();
    let result = detect_sync(temp_dir.path());
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
}

#[test]
fn test_detect_package_manager_from_npm_shrinkwrap() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(
        temp_dir
            .path()
            .join("npm-shrinkwrap.json")
            .to_str()
            .unwrap(),
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_json() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(temp_dir.path().join("nirs.json").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("nirs.json"),
        r#"{
        "default_package_manager": "yarn"
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Yarn));
    std::env::remove_var("HOME");
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_yaml() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(temp_dir.path().join("nirs.yaml").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("nirs.yaml"),
        r#"
        default_package_manager: "pnpm"
    "#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    std::env::remove_var("HOME");
    Ok(())
}

#[test]
fn test_detect_package_manager_from_package_json_invalid_pm() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(temp_dir.path().join("package.json").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
        "packageManager": 123
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}

#[test]
fn test_detect_sync_no_package_manager_npm_in_path() {
    let temp_dir = tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&bin_dir).unwrap();
    File::create(bin_dir.join("npm")).unwrap();

    std::env::set_var("PATH", bin_dir.to_str().unwrap());
    let result = detect_sync(temp_dir.path());
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    std::env::remove_var("PATH");
}

#[test]
fn test_detect_package_manager_from_package_json_valid_version() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(temp_dir.path().join("package.json").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
        "packageManager": "npm@8.0.0"
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}

#[test]
fn test_detect_package_manager_from_package_json_no_version() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(temp_dir.path().join("package.json").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("package.json"),
        r#"{
        "packageManager": "npm"
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}

#[test]
fn test_detect_package_manager_from_package_json_missing() -> Result<()> {
    let temp_dir = tempdir()?;
    create_empty_file(temp_dir.path().join("package.json").to_str().unwrap())?;
    std::fs::write(temp_dir.path().join("package.json"), r#"{}"#)?;
    let result = detect(temp_dir.path())?;
    std::env::set_var("HOME", temp_dir.path());
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    std::env::remove_var("HOME");
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_toml() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(temp_dir.path().join("nirs.toml").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("nirs.toml"),
        r#"default_package_manager = "bun""#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Bun));
    std::env::remove_var("HOME");
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_json_home() -> Result<()> {
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    let config_dir = home_dir.path().join(".config");
    fs::create_dir_all(&config_dir)?;

    std::env::set_var("HOME", home_dir.path());
    env::set_var("PATH", "");
    create_empty_file(config_dir.join("nirs.json").to_str().unwrap())?;
    std::fs::write(
        config_dir.join("nirs.json"),
        r#"{
        "default_package_manager": "pnpm"
    }"#,
    )?;

    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    std::env::remove_var("PATH");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_yaml_home() -> Result<()> {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".config");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("nirs.yaml");
    fs::write(config_path, "default_package_manager: yarn").unwrap();

    env::set_var("HOME", dir.path());
    env::set_var("PATH", "");
    let result = detect(dir.path())?;
    std::env::remove_var("HOME");
    std::env::remove_var("PATH");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Yarn));
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_toml_home() -> Result<()> {
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    std::env::set_var("HOME", home_dir.path());
    create_empty_file(home_dir.path().join("nirs.toml").to_str().unwrap())?;
    std::fs::write(
        home_dir.path().join("nirs.toml"),
        r#"default_package_manager = "npm""#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    std::env::remove_var("HOME");
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_toml_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("nirs.toml");
    fs::write(config_path, "default_package_manager = 'invalid'").unwrap();

    env::set_var("PATH", "");
    assert_eq!(detect(dir.path()).unwrap(), None);
}

#[test]
fn test_detect_package_manager_from_nirs_json_invalid() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(temp_dir.path().join("nirs.json").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("nirs.json"),
        r#"{
        "default_package_manager": 123
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, None);
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_yaml_invalid() -> Result<()> {
    let temp_dir = tempdir()?;
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(temp_dir.path().join("nirs.yaml").to_str().unwrap())?;
    std::fs::write(
        temp_dir.path().join("nirs.yaml"),
        r#"
        default_package_manager: 123
    "#,
    )?;
    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, None);
    Ok(())
}

#[test]
fn test_detect_package_manager_from_nirs_toml_home_missing() -> Result<()> {
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    std::env::set_var("HOME", home_dir.path());
    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}
