use anyhow::{Context, Result};
use log::{info, warn};
use reqwest;
use serde::Deserialize;
use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime},
};

const GITHUB_API_URL: &str = "https://api.github.com/repos/spa5k/nirs/releases/latest";
const LAST_UPDATE_CHECK_FILE: &str = ".nirs_last_update_check";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
}

async fn fetch_latest_version() -> Result<String> {
    info!(
        "Fetching latest version from GitHub API: {}",
        GITHUB_API_URL
    );
    let client = reqwest::Client::new();
    let response = client
        .get(GITHUB_API_URL)
        .header("User-Agent", "nirs-update-checker") // GitHub requires a User-Agent header
        .send()
        .await
        .context("Failed to send request to GitHub API")?;

    if !response.status().is_success() {
        warn!(
            "GitHub API request failed with status: {}",
            response.status()
        );
        return Err(anyhow::anyhow!("GitHub API request failed"));
    }

    let release: GithubRelease = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    info!("Latest version from GitHub: {}", release.tag_name);
    Ok(release.tag_name)
}

fn get_current_version() -> Result<String> {
    info!("Getting current version of nirs");
    let output =
        std::process::Command::new(std::env::var("CARGO_BIN_NAME").unwrap_or("nirs".to_string()))
            .arg("--version")
            .output()
            .context("Failed to execute 'nirs --version'")?;

    if !output.status.success() {
        warn!(
            "'nirs --version' failed with exit code: {:?}",
            output.status.code()
        );
        return Err(anyhow::anyhow!("Failed to get current version"));
    }

    let version = String::from_utf8(output.stdout)
        .context("Failed to parse 'nirs --version' output")?
        .trim()
        .to_string();

    info!("Current version: {}", version);
    Ok(version)
}

fn should_check_for_update() -> bool {
    let home_dir = match std::env::var("HOME") {
        Ok(path) => path,
        Err(_) => {
            warn!("Could not determine home directory. Skipping daily update check.");
            return false;
        }
    };
    let last_update_check_path = Path::new(&home_dir).join(LAST_UPDATE_CHECK_FILE);

    let last_check_time = match fs::metadata(&last_update_check_path) {
        Ok(metadata) => match metadata.modified() {
            Ok(time) => time,
            Err(_) => {
                warn!("Could not read last update check time. Performing update check.");
                return true;
            }
        },
        Err(_) => {
            info!("No previous update check found. Performing update check.");
            return true;
        }
    };

    let now = SystemTime::now();
    let duration_since_last_check = match now.duration_since(last_check_time) {
        Ok(duration) => duration,
        Err(_) => {
            warn!("Clock may have gone backwards. Performing update check.");
            return true;
        }
    };

    if duration_since_last_check > Duration::from_secs(24 * 60 * 60) {
        info!("Last update check was more than 24 hours ago.");
        return true;
    }

    info!("Last update check was less than 24 hours ago. Skipping update check.");
    false
}

fn update_last_check_time() -> Result<()> {
    let home_dir = match std::env::var("HOME") {
        Ok(path) => path,
        Err(_) => {
            warn!("Could not determine home directory. Skipping update check time update.");
            return Ok(());
        }
    };
    let last_update_check_path = Path::new(&home_dir).join(LAST_UPDATE_CHECK_FILE);

    fs::write(last_update_check_path, "").context("Failed to update last update check time")?;

    Ok(())
}

pub async fn check_for_update() -> Result<()> {
    if !should_check_for_update() {
        return Ok(());
    }

    let latest_version = match fetch_latest_version().await {
        Ok(version) => version,
        Err(e) => {
            warn!("Failed to fetch latest version: {}", e);
            return Ok(());
        }
    };

    let current_version = match get_current_version() {
        Ok(version) => version,
        Err(e) => {
            warn!("Failed to get current version: {}", e);
            return Ok(());
        }
    };

    if latest_version != current_version {
        info!(
            "New version available: {} (current: {})",
            latest_version, current_version
        );

        // TODO: Detect installation method and provide appropriate update instructions
        println!("A new version of nirs is available: {}.  You can update by running `cargo install nirs`", latest_version);
    } else {
        info!("nirs is up to date.");
    }

    if let Err(e) = update_last_check_time() {
        warn!("Failed to update last check time: {}", e);
    }

    Ok(())
}
