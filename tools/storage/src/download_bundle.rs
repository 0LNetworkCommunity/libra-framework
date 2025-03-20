use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use anyhow::bail;

#[derive(Deserialize)]
struct GitHubContent {
    download_url: Option<String>,
    #[serde(rename = "type")]
    content_type: String,
    name: String,
}

/// Finds the closest matching epoch folder name for a given epoch number
pub async fn find_closest_epoch_folder(
    client: &Client,
    owner: &str,
    repo: &str,
    branch: &str,
    target_epoch: u64,
) -> Result<String> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/contents/snapshots?ref={}",
        owner, repo, branch
    );

    let contents: Vec<GitHubContent> = client
        .get(&api_url)
        .header("User-Agent", "libra-framework-downloader")
        .send()
        .await
        .context("Failed to list snapshots directory")?
        .json()
        .await
        .context("Failed to parse snapshots directory contents")?;

    // Filter and parse epoch numbers from folder names
    let mut epoch_folders: Vec<(u64, String)> = contents
        .into_iter()
        .filter_map(|item| {
            if item.content_type == "dir" && item.name.starts_with("epoch_ending_") {
                let epoch_str = item.name.strip_prefix("epoch_ending_")?;
                if let Ok(epoch) = u64::from_str(epoch_str) {
                    return Some((epoch, item.name));
                }
            }
            None
        })
        .collect();

    if epoch_folders.is_empty() {
        bail!("No epoch folders found in snapshots directory");
    }

    // Sort by epoch number
    epoch_folders.sort_by_key(|(epoch, _)| *epoch);

    // Find the closest epoch
    let closest = epoch_folders
        .into_iter()
        .min_by_key(|(epoch, _)| {
            if *epoch > target_epoch {
                *epoch - target_epoch
            } else {
                target_epoch - *epoch
            }
        })
        .context("Failed to find closest epoch")?;

    println!("Found closest epoch folder: {} for target epoch {}", closest.1, target_epoch);
    Ok(closest.1)
}

pub async fn download_github_folder(
    owner: &str,
    repo: &str,
    path: &str,
    branch: &str,
    output_dir: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut pending_dirs = vec![(path.to_string(), output_dir.to_string())];

    while let Some((current_path, current_dir)) = pending_dirs.pop() {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            owner, repo, current_path, branch
        );

        println!("Downloading from: {}", api_url);

        let contents: Vec<GitHubContent> = client
            .get(&api_url)
            .header("User-Agent", "libra-framework-downloader")
            .send()
            .await
            .context("Failed to send request")?
            .json()
            .await
            .context("Failed to parse JSON response")?;

        fs::create_dir_all(&current_dir)?;

        for item in contents {
            let output_path = Path::new(&current_dir).join(&item.name);

            if item.content_type == "file" {
                if let Some(download_url) = item.download_url {
                    println!("Downloading file: {}", item.name);
                    let content = client
                        .get(&download_url)
                        .header("User-Agent", "libra-framework-downloader")
                        .send()
                        .await?
                        .bytes()
                        .await?;

                    fs::write(&output_path, content)
                        .with_context(|| format!("Failed to write file: {}", item.name))?;
                }
            } else if item.content_type == "dir" {
                println!("Processing directory: {}", item.name);
                let new_path = format!("{}/{}", current_path, item.name);
                pending_dirs.push((
                    new_path,
                    output_path.to_str().unwrap().to_string(),
                ));
            }
        }
    }

    Ok(())
}
