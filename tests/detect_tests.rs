use std::env;
use std::fs;
use std::fs::File;

use anyhow::Result;
use tempfile::tempdir;

use nirs::detect::{detect, detect_sync, PackageManagerFactoryEnum};

mod common;

use common::create_empty_file;

#[test]
fn test_detect_yarn_from_package_json() -> Result<()> {
    // Test that yarn is detected when packageManager field in package.json specifies yarn
    let temp_dir = tempdir()?;
    let package_json_path = temp_dir.path().join("package.json");
    // if file is found, delete it
    if fs::metadata(&package_json_path).is_ok() {
        fs::remove_file(&package_json_path)?;
    }
    create_empty_file(package_json_path.to_str().unwrap())?;
    std::fs::write(
        package_json_path,
        r#"{
        "packageManager": "yarn@1.22.0"
    }"#,
    )?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Yarn));
    Ok(())
}

#[test]
fn test_detect_yarn_from_yarn_lockfile() -> Result<()> {
    // Test that yarn is detected when yarn.lock file exists
    let temp_dir = tempdir()?;
    let yarn_lock_path = temp_dir.path().join("yarn.lock");
    // if file is found, delete it
    if fs::metadata(&yarn_lock_path).is_ok() {
        fs::remove_file(&yarn_lock_path)?;
    }
    create_empty_file(yarn_lock_path.to_str().unwrap())?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Yarn));
    Ok(())
}

#[test]
fn test_detect_returns_none_for_invalid_directory() -> Result<()> {
    // Test that detect returns None when the directory does not exist
    let temp_dir = tempdir()?;
    let invalid_path = temp_dir.path().join("invalid");
    let result = detect(&invalid_path)?;
    assert_eq!(result, None);
    Ok(())
}

#[test]
fn test_detect_sync_returns_none_when_no_package_manager_is_found() {
    // Save original PATH
    let original_path = env::var("PATH").ok();

    // Create temp directory and set empty PATH
    let dir = tempdir().unwrap();
    env::set_var("PATH", "");

    // Run the test
    let result = detect_sync(dir.path());

    // Restore original PATH
    if let Some(path) = original_path {
        env::set_var("PATH", path);
    } else {
        env::remove_var("PATH");
    }

    assert_eq!(result, None);
}

#[test]
fn test_detect_sync_npm_from_package_json() {
    // Test that detect_sync detects npm when packageManager field in package.json specifies npm
    let temp_dir = tempdir().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    // if file is found, delete it
    if fs::metadata(&package_json_path).is_ok() {
        fs::remove_file(&package_json_path).unwrap();
    }
    create_empty_file(package_json_path.to_str().unwrap()).unwrap();
    std::fs::write(
        package_json_path,
        r#"{
        "packageManager": "npm@8.0.0"
    }"#,
    )
    .unwrap();
    let result = detect_sync(temp_dir.path());
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
}

#[test]
fn test_detect_npm_from_npm_shrinkwrap() -> Result<()> {
    // Test that npm is detected when npm-shrinkwrap.json file exists
    let temp_dir = tempdir()?;
    let shrinkwrap_path = temp_dir.path().join("npm-shrinkwrap.json");
    // if file is found, delete it
    if fs::metadata(&shrinkwrap_path).is_ok() {
        fs::remove_file(&shrinkwrap_path)?;
    }
    create_empty_file(shrinkwrap_path.to_str().unwrap())?;
    let result = detect(temp_dir.path())?;
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}

#[test]
fn test_detect_yarn_from_nirs_json() -> Result<()> {
    // Test that yarn is detected when default_package_manager in nirs.json specifies yarn
    let temp_dir = tempdir()?;
    let nirs_json_path = temp_dir.path().join("nirs.json");
    // if file is found, delete it
    if fs::metadata(&nirs_json_path).is_ok() {
        fs::remove_file(&nirs_json_path)?;
    }
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(nirs_json_path.to_str().unwrap())?;
    std::fs::write(
        nirs_json_path,
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
fn test_detect_pnpm_from_nirs_yaml() -> Result<()> {
    // Test that pnpm is detected when default_package_manager in nirs.yaml specifies pnpm
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    let config_dir = home_dir.path().join(".config");
    fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("nirs.yaml");

    // if file is found, delete it
    if fs::metadata(&config_file).is_ok() {
        fs::remove_file(&config_file)?;
    }

    std::env::set_var("HOME", home_dir.path());
    create_empty_file(config_file.to_str().unwrap())?;
    std::fs::write(
        config_file,
        r#"
default_package_manager: "pnpm"
"#,
    )?;
    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    Ok(())
}

#[test]
fn test_detect_npm_from_package_json_with_invalid_pm_value() -> Result<()> {
    // Test that npm is detected when packageManager in package.json is invalid
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
fn test_detect_sync_npm_from_path() {
    // Test that detect_sync detects npm when npm is found in PATH
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
fn test_detect_npm_from_package_json_with_valid_version() -> Result<()> {
    // Test that npm is detected when packageManager in package.json has a valid version
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
fn test_detect_npm_from_package_json_without_version() -> Result<()> {
    // Test that npm is detected when packageManager in package.json has no version
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
fn test_detect_npm_when_package_json_is_missing_packagemanager_field() -> Result<()> {
    // Test that npm is detected when package.json is missing the packageManager field
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
fn test_detect_bun_from_nirs_toml() -> Result<()> {
    // Test that bun is detected when default_package_manager in nirs.toml specifies bun
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
fn test_detect_pnpm_from_nirs_json_in_home_config() -> Result<()> {
    // Test that pnpm is detected when default_package_manager in nirs.json in home config specifies pnpm
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    let config_dir = home_dir.path().join(".config");
    fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("nirs.json");

    // if file is found, delete it
    if fs::metadata(&config_file).is_ok() {
        fs::remove_file(&config_file)?;
    }

    std::env::set_var("HOME", home_dir.path());
    create_empty_file(config_file.to_str().unwrap())?;
    std::fs::write(
        config_file,
        r#"{
        "default_package_manager": "pnpm"
    }"#,
    )?;

    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Pnpm));
    Ok(())
}

#[test]
fn test_detect_yarn_from_nirs_yaml_in_home_config() -> Result<()> {
    // Test that yarn is detected when default_package_manager in nirs.yaml in home config specifies yarn
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
fn test_detect_npm_from_nirs_toml_in_home_config() -> Result<()> {
    // Test that npm is detected when default_package_manager in nirs.toml in home config specifies npm
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    let config_dir = home_dir.path().join(".config");
    fs::create_dir_all(&config_dir)?;
    let config_file = config_dir.join("nirs.toml");

    // if file is found, delete it
    if fs::metadata(&config_file).is_ok() {
        fs::remove_file(&config_file)?;
    }

    std::env::set_var("HOME", home_dir.path());
    create_empty_file(config_file.to_str().unwrap())?;
    std::fs::write(config_file, r#"default_package_manager = "npm""#)?;
    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}

#[test]
fn test_detect_returns_none_when_nirs_toml_has_invalid_package_manager() {
    // Test that detect returns None when nirs.toml has an invalid package manager
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("nirs.toml");
    // if file is found, delete it
    if fs::metadata(config_path.clone()).is_ok() {
        fs::remove_file(config_path.clone()).unwrap();
    }
    fs::write(config_path.clone(), "default_package_manager = 'invalid'").unwrap();

    env::set_var("PATH", "");
    assert_eq!(detect(dir.path()).unwrap(), None);
}

#[test]
fn test_detect_returns_none_when_nirs_json_has_invalid_package_manager() -> Result<()> {
    // Test that detect returns None when nirs.json has an invalid package manager
    let temp_dir = tempdir()?;
    let nirs_json_path = temp_dir.path().join("nirs.json");
    // if file is found, delete it
    if fs::metadata(&nirs_json_path).is_ok() {
        fs::remove_file(&nirs_json_path)?;
    }
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(nirs_json_path.to_str().unwrap())?;
    std::fs::write(
        nirs_json_path,
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
fn test_detect_returns_none_when_nirs_yaml_has_invalid_package_manager() -> Result<()> {
    // Test that detect returns None when nirs.yaml has an invalid package manager
    let temp_dir = tempdir()?;
    let nirs_yaml_path = temp_dir.path().join("nirs.yaml");
    // if file is found, delete it
    if fs::metadata(&nirs_yaml_path).is_ok() {
        fs::remove_file(&nirs_yaml_path)?;
    }
    std::env::set_var("HOME", temp_dir.path());
    create_empty_file(nirs_yaml_path.to_str().unwrap())?;
    std::fs::write(
        nirs_yaml_path,
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
fn test_detect_npm_when_nirs_toml_is_missing_in_home_config() -> Result<()> {
    // Test that npm is detected when nirs.toml is missing in home config
    let temp_dir = tempdir()?;
    let home_dir = tempdir()?;
    std::env::set_var("HOME", home_dir.path());
    let result = detect(temp_dir.path())?;
    std::env::remove_var("HOME");
    assert_eq!(result, Some(PackageManagerFactoryEnum::Npm));
    Ok(())
}
