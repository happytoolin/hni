use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use configparser::ini::Ini;

use super::{
    error::{HniError, HniResult},
    types::PackageManager,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultAgent {
    Prompt,
    Agent(PackageManager),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunAgent {
    PackageManager,
    Node,
}

#[derive(Debug, Clone)]
pub struct HniConfig {
    pub default_agent: DefaultAgent,
    pub global_agent: PackageManager,
    pub run_agent: RunAgent,
    pub native_mode: bool,
    pub use_sfw: bool,
    pub auto_install: bool,
    pub config_path: Option<PathBuf>,
}

impl Default for HniConfig {
    fn default() -> Self {
        Self {
            default_agent: DefaultAgent::Prompt,
            global_agent: PackageManager::Npm,
            run_agent: RunAgent::PackageManager,
            native_mode: false,
            use_sfw: false,
            auto_install: false,
            config_path: None,
        }
    }
}

impl HniConfig {
    pub fn load() -> HniResult<Self> {
        let mut cfg = Self::default();

        let explicit_path = env::var("HNI_CONFIG_FILE").ok().map(PathBuf::from);

        if let Some(path) = explicit_path {
            let loaded = parse_hnirc_file(&path, &mut cfg, true).map_err(|error| {
                error.with_context(format!("failed to load {}", path.display()))
            })?;
            if loaded {
                cfg.config_path = Some(path);
            }
        } else if let Some(path) = default_config_path() {
            let loaded = parse_hnirc_file(&path, &mut cfg, false).map_err(|error| {
                error.with_context(format!("failed to load {}", path.display()))
            })?;
            if loaded {
                cfg.config_path = Some(path);
            }
        }

        if let Ok(v) = env::var("HNI_DEFAULT_AGENT") {
            cfg.default_agent = parse_default_agent(&v)?;
        }

        if let Ok(v) = env::var("HNI_GLOBAL_AGENT") {
            cfg.global_agent = parse_pm(&v)?;
        }

        if let Ok(v) = env::var("HNI_USE_SFW") {
            cfg.use_sfw = parse_bool(&v)?;
        }

        if let Ok(v) = env::var("HNI_NATIVE") {
            cfg.native_mode = parse_bool(&v)?;
        }

        if let Ok(v) = env::var("HNI_AUTO_INSTALL") {
            cfg.auto_install = parse_bool(&v)?;
        }

        Ok(cfg)
    }
}

fn default_config_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let hni = home.join(".hnirc");
    hni.exists().then_some(hni)
}

fn parse_hnirc_file(path: &Path, config: &mut HniConfig, required: bool) -> HniResult<bool> {
    let raw = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound && !required => return Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound && required => {
            return Err(HniError::config(format!(
                "config file not found: {}",
                path.display()
            )));
        }
        Err(error) => {
            return Err(HniError::config(format!(
                "failed to read config file {}: {error}",
                path.display()
            )));
        }
    };

    let mut ini = Ini::new();
    ini.read(raw).map_err(|error| {
        HniError::config(format!("failed to parse {}: {error}", path.display()))
    })?;

    if let Some(v) = ini.get("default", "defaultagent") {
        config.default_agent = parse_default_agent(v.trim())?;
    }
    if let Some(v) = ini.get("default", "globalagent") {
        config.global_agent = parse_pm(v.trim())?;
    }
    if let Some(v) = ini.get("default", "runagent") {
        config.run_agent = parse_run_agent(v.trim())?;
    }
    if let Some(v) = ini.get("default", "nativemode") {
        config.native_mode = parse_bool(v.trim())?;
    }
    if let Some(v) = ini.get("default", "usesfw") {
        config.use_sfw = parse_bool(v.trim())?;
    }

    Ok(true)
}

fn parse_pm(value: &str) -> HniResult<PackageManager> {
    PackageManager::from_name(&value.to_ascii_lowercase())
        .ok_or_else(|| HniError::config(format!("unsupported package manager: {value}")))
}

fn parse_default_agent(value: &str) -> HniResult<DefaultAgent> {
    if value.eq_ignore_ascii_case("prompt") {
        return Ok(DefaultAgent::Prompt);
    }
    Ok(DefaultAgent::Agent(parse_pm(value)?))
}

fn parse_run_agent(value: &str) -> HniResult<RunAgent> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "node" => Ok(RunAgent::Node),
        _ if PackageManager::from_name(&normalized).is_some() => Ok(RunAgent::PackageManager),
        _ => Err(HniError::config(format!("invalid runAgent value: {value}"))),
    }
}

fn parse_bool(value: &str) -> HniResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(HniError::config(format!("invalid boolean: {value}"))),
    }
}

#[cfg(test)]
fn default_config_path_with_home(home: &Path) -> Option<PathBuf> {
    let hni = home.join(".hnirc");
    hni.exists().then_some(hni)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_hnirc_values() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".hnirc");
        fs::write(
            &path,
            "defaultAgent=pnpm\nglobalAgent=yarn\nrunAgent=node\nuseSfw=true\n",
        )
        .unwrap();

        let mut cfg = HniConfig::default();
        parse_hnirc_file(&path, &mut cfg, true).unwrap();

        assert_eq!(cfg.default_agent, DefaultAgent::Agent(PackageManager::Pnpm));
        assert_eq!(cfg.global_agent, PackageManager::Yarn);
        assert_eq!(cfg.run_agent, RunAgent::Node);
        assert!(!cfg.native_mode);
        assert!(cfg.use_sfw);
        assert!(!cfg.auto_install);
    }

    #[test]
    fn parse_default_prompt() {
        let parsed = parse_default_agent("prompt").unwrap();
        assert_eq!(parsed, DefaultAgent::Prompt);
    }

    #[test]
    fn explicit_missing_config_is_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing-hnirc");
        let mut cfg = HniConfig::default();
        let err = parse_hnirc_file(&path, &mut cfg, true).unwrap_err();
        assert!(err.to_string().contains("config file not found"));
    }

    #[test]
    fn default_config_path_only_considers_hnirc() {
        let dir = tempdir().unwrap();
        let home = dir.path();
        fs::write(home.join(".nirc"), "defaultAgent=pnpm\n").unwrap();
        fs::write(home.join(".hnirc"), "defaultAgent=bun\n").unwrap();

        let resolved = default_config_path_with_home(home).unwrap();
        assert_eq!(resolved, home.join(".hnirc"));
    }

    #[test]
    fn default_config_path_ignores_nirc_when_hnirc_missing() {
        let dir = tempdir().unwrap();
        let home = dir.path();
        fs::write(home.join(".nirc"), "defaultAgent=pnpm\n").unwrap();

        assert_eq!(default_config_path_with_home(home), None);
    }
}
