use std::{collections::BTreeSet, path::Path};

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};

use crate::core::pkg_json::read_package_json;

pub fn choose_dependencies_for_uninstall(cwd: &Path) -> Result<Vec<String>> {
    let Some(pkg) = read_package_json(cwd)? else {
        return Err(anyhow!("package.json not found"));
    };

    let mut deps = BTreeSet::new();
    for map in [
        pkg.dependencies,
        pkg.dev_dependencies,
        pkg.peer_dependencies,
        pkg.optional_dependencies,
    ]
    .into_iter()
    .flatten()
    {
        deps.extend(map.into_keys());
    }

    if deps.is_empty() {
        return Err(anyhow!("no dependencies found in package.json"));
    }

    let items: Vec<String> = deps.into_iter().collect();
    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select dependencies to remove")
        .items(&items)
        .interact()?;

    if selected.is_empty() {
        return Ok(Vec::new());
    }

    Ok(selected.into_iter().map(|i| items[i].clone()).collect())
}
