use inquire::Select;
use anyhow::Result;
use crate::fetch::fetch_npm_packages;

pub async fn interactive_install() -> Result<()> {
    let packages = fetch_npm_packages("react").await?;
    
    let package_choice = Select::new("Choose a package:", packages)
        .with_page_size(15)
        .prompt()?;
    
    let mode = Select::new("Install as:", vec!["prod", "dev", "peer"])
        .prompt()?;
    
    // Build command args based on selections
    Ok(())
}
