use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::core::resolve::ResolveContext;

#[derive(Debug, Deserialize)]
struct NpmResponse {
    objects: Vec<NpmEntry>,
}

#[derive(Debug, Deserialize)]
struct NpmEntry {
    package: NpmPackage,
}

#[derive(Debug, Deserialize)]
struct NpmPackage {
    name: String,
    version: String,
    description: Option<String>,
}

pub fn augment_ni_args_interactive(args: Vec<String>, ctx: &ResolveContext) -> Result<Vec<String>> {
    let mut args = args;
    let is_interactive = args.first().is_some_and(|a| a == "-i");
    if !is_interactive {
        return Ok(args);
    }

    let pattern = args
        .get(1)
        .filter(|v| !v.starts_with('-'))
        .cloned()
        .map_or_else(prompt_pattern, Ok)?;

    if pattern.trim().is_empty() {
        return Err(anyhow!(
            "interactive install requires a package search pattern"
        ));
    }

    let packages = fetch_npm_packages(&pattern)?;
    if packages.is_empty() {
        return Err(anyhow!("no npm packages found for pattern '{pattern}'"));
    }

    let labels: Vec<String> = packages
        .iter()
        .map(|pkg| {
            let description = pkg.description.as_deref().unwrap_or("");
            format!("{}  v{}  {}", pkg.name, pkg.version, description)
        })
        .collect();

    let idx = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose package to install")
        .items(&labels)
        .default(0)
        .interact_opt()?
        .ok_or_else(|| anyhow!("package selection canceled"))?;

    let chosen = &packages[idx].name;

    let agent = crate::core::resolve::detected_package_manager(ctx)?;
    let can_peer = matches!(
        agent,
        crate::core::types::PackageManager::Npm | crate::core::types::PackageManager::Pnpm
    );
    let mode_labels = if can_peer {
        vec!["prod", "dev", "peer"]
    } else {
        vec!["prod", "dev"]
    };
    let mode_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Install {chosen} as"))
        .items(&mode_labels)
        .default(0)
        .interact()?;

    args.retain(|arg| !matches!(arg.as_str(), "-i" | "-d" | "-p"));
    args.push(chosen.clone());

    match mode_labels[mode_idx] {
        "dev" => args.push("-D".to_string()),
        "peer" => args.push("--save-peer".to_string()),
        _ => {}
    }

    Ok(args)
}

fn prompt_pattern() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("search for package")
        .interact_text()
        .context("failed to read interactive search pattern")
}

fn fetch_npm_packages(pattern: &str) -> Result<Vec<NpmPackage>> {
    let url = format!("https://registry.npmjs.com/-/v1/search?text={pattern}&size=35");

    let response = Client::new()
        .get(url)
        .send()
        .context("failed to query npm registry")?
        .error_for_status()
        .context("npm registry returned an error")?;

    let parsed = response
        .json::<NpmResponse>()
        .context("failed to parse npm registry response")?;

    Ok(parsed.objects.into_iter().map(|obj| obj.package).collect())
}
