use inquire::Select;
use anyhow::Result;
use crate::fetch::fetch_npm_packages;
use crate::detect::{detect, PackageManager};
use std::path::Path;
use crate::parse_ni;
use crate::execute;

pub async fn interactive_install() -> Result<()> {
    let packages = fetch_npm_packages("react").await?;
    
    let package_choice = Select::new("Choose a package:", packages)
        .with_page_size(15)
        .prompt()?;
    
    let mode = Select::new("Install as:", vec!["prod", "dev", "peer"])
        .prompt()?;

    let current_dir = std::env::current_dir()?;
    let package_manager = detect(Path::new(&current_dir)).unwrap_or(PackageManager::Npm);

    let install_args = vec![package_choice.to_string()];
    let command = parse_ni(package_manager, &install_args);

    log::info!("Executing command: {:?}", command);
    let current_dir = std::env::current_dir()?;
    execute(command, Path::new(&current_dir))?;
    
    // Build command args based on selections
    Ok(())
}
