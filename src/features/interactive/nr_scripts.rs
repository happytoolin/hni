use std::{collections::BTreeMap, path::Path};

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use crate::core::pkg_json::read_package_json;

pub fn read_scripts(cwd: &Path) -> Result<Vec<ScriptEntry>> {
    let Some(pkg) = read_package_json(cwd)? else {
        return Ok(Vec::new());
    };

    let scripts = pkg.scripts.unwrap_or_default();
    let scripts_info = pkg.scripts_info.unwrap_or_default();

    Ok(build_script_entries(&scripts, &scripts_info))
}

pub fn choose_script_interactive(cwd: &Path) -> Result<String> {
    let scripts = read_scripts(cwd)?;
    if scripts.is_empty() {
        return Err(anyhow!("no scripts found in package.json"));
    }

    let labels: Vec<String> = scripts
        .iter()
        .map(|entry| format!("{}: {}", entry.name, entry.description))
        .collect();

    let idx = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose script to run")
        .items(&labels)
        .default(0)
        .interact_opt()?
        .ok_or_else(|| anyhow!("script selection canceled"))?;

    Ok(scripts[idx].name.clone())
}

#[derive(Debug, Clone)]
pub struct ScriptEntry {
    pub name: String,
    pub description: String,
}

fn build_script_entries(
    scripts: &BTreeMap<String, String>,
    scripts_info: &BTreeMap<String, String>,
) -> Vec<ScriptEntry> {
    scripts
        .iter()
        .filter(|(name, _)| !name.starts_with('?'))
        .map(|(name, cmd)| {
            let description = scripts_info
                .get(name)
                .cloned()
                .or_else(|| scripts.get(&format!("?{name}")).cloned())
                .unwrap_or_else(|| cmd.clone());

            ScriptEntry {
                name: name.clone(),
                description,
            }
        })
        .collect()
}
