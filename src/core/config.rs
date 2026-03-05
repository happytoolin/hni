use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use configparser::ini::Ini;

use super::types::PackageManager;

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
            use_sfw: false,
            auto_install: false,
            config_path: None,
        }
    }
}

impl HniConfig {
    pub fn load() -> Result<Self> {
        let mut cfg = Self::default();

        let config_path = env_key(&["HNI_CONFIG_FILE", "NI_CONFIG_FILE"])
            .map(PathBuf::from)
            .or_else(default_config_path);

        if let Some(path) = config_path {
            parse_hnirc_file(&path, &mut cfg)
                .with_context(|| format!("failed to load config from {}", path.display()))?;
            cfg.config_path = Some(path);
        }

        if let Some(v) = env_key(&["HNI_DEFAULT_AGENT", "NI_DEFAULT_AGENT"]) {
            cfg.default_agent = parse_default_agent(&v)?;
        }

        if let Some(v) = env_key(&["HNI_GLOBAL_AGENT", "NI_GLOBAL_AGENT"]) {
            cfg.global_agent = parse_pm(&v)?;
        }

        if let Some(v) = env_key(&["HNI_USE_SFW", "NI_USE_SFW"]) {
            cfg.use_sfw = parse_bool(&v)?;
        }

        if let Some(v) = env_key(&["HNI_AUTO_INSTALL", "NI_AUTO_INSTALL"]) {
            cfg.auto_install = parse_bool(&v)?;
        }

        Ok(cfg)
    }
}

fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        let hni = home.join(".hnirc");
        if hni.exists() {
            hni
        } else {
            home.join(".nirc")
        }
    })
}

fn env_key(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| env::var(key).ok())
}

fn parse_hnirc_file(path: &Path, config: &mut HniConfig) -> Result<()> {
    let raw = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(e)
                .with_context(|| format!("failed to read config file {}", path.display()))?
        }
    };

    let mut ini = Ini::new();
    ini.read(raw)
        .map_err(|e| anyhow!("failed to parse config file {}: {e}", path.display()))?;

    if let Some(v) = ini.get("default", "defaultagent") {
        config.default_agent = parse_default_agent(v.trim())?;
    }
    if let Some(v) = ini.get("default", "globalagent") {
        config.global_agent = parse_pm(v.trim())?;
    }
    if let Some(v) = ini.get("default", "runagent") {
        config.run_agent = parse_run_agent(v.trim())?;
    }
    if let Some(v) = ini.get("default", "usesfw") {
        config.use_sfw = parse_bool(v.trim())?;
    }
    if let Some(v) = ini.get("default", "autoinstall") {
        config.auto_install = parse_bool(v.trim())?;
    }

    Ok(())
}

fn parse_pm(value: &str) -> Result<PackageManager> {
    PackageManager::from_name(&value.to_ascii_lowercase())
        .ok_or_else(|| anyhow!("unsupported package manager: {value}"))
}

fn parse_default_agent(value: &str) -> Result<DefaultAgent> {
    if value.eq_ignore_ascii_case("prompt") {
        return Ok(DefaultAgent::Prompt);
    }
    Ok(DefaultAgent::Agent(parse_pm(value)?))
}

fn parse_run_agent(value: &str) -> Result<RunAgent> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "node" => Ok(RunAgent::Node),
        _ if PackageManager::from_name(&normalized).is_some() => Ok(RunAgent::PackageManager),
        _ => Err(anyhow!("invalid runAgent value: {value}")),
    }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(anyhow!("invalid boolean: {value}")),
    }
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
            "defaultAgent=pnpm\nglobalAgent=yarn\nrunAgent=node\nuseSfw=true\nautoInstall=true\n",
        )
        .unwrap();

        let mut cfg = HniConfig::default();
        parse_hnirc_file(&path, &mut cfg).unwrap();

        assert_eq!(cfg.default_agent, DefaultAgent::Agent(PackageManager::Pnpm));
        assert_eq!(cfg.global_agent, PackageManager::Yarn);
        assert_eq!(cfg.run_agent, RunAgent::Node);
        assert!(cfg.use_sfw);
        assert!(cfg.auto_install);
    }

    #[test]
    fn parse_default_prompt() {
        let parsed = parse_default_agent("prompt").unwrap();
        assert_eq!(parsed, DefaultAgent::Prompt);
    }
}
