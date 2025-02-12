use inquire::Select;
use anyhow::Result;
use crate::fetch::fetch_npm_packages;
use crate::detect::{detect, PackageManager, resolve_command};
use std::path::Path;

pub async fn interactive_install() -> Result<()> {
    let packages = fetch_npm_packages("react").await?;
    
    let package_choice = Select::new("Choose a package:", packages)
        .with_page_size(15)
        .prompt()?;
    
    let mode = Select::new("Install as:", vec!["prod", "dev", "peer"])
        .prompt()?;

    let current_dir = std::env::current_dir()?;
    let package_manager = detect(Path::new(&current_dir)).unwrap_or(PackageManager::Npm);

    let install_args = vec![package_choice.as_str()];
    let command = resolve_command(package_manager, "add", install_args.clone());

    match command {
        Some(cmd) => {
            println!("Executing command: {:?}", cmd);
            // Execute the command here
        }
        None => {
            println!("Command not found for the selected package manager.");
        }
    }
    
    // Build command args based on selections
    Ok(())
}
