use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NiConfig {
    #[serde(default = "default_agent")]
    pub default_agent: String,
    #[serde(default = "global_agent")]
    pub global_agent: String,
}

fn default_agent() -> String { "prompt".into() }
fn global_agent() -> String { "npm".into() }

impl NiConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut settings = Config::new();
        // Optionally load from "~/.nirc"
        // settings.merge(config::File::with_name("~/.nirc")).ok();
        // Merge environment variables prefixed with NI
        // settings.merge(config::Environment::with_prefix("NI")).ok();
        settings.try_into().map_err(Into::into)
    }
}
