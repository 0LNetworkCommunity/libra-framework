use crate::parse_folder_names::{
    parse_epoch_ending_number, parse_state_epoch_info, parse_transaction_number,
};
use anyhow::bail;
use anyhow::{Context, Result};
use diem_logger::info;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// Update GitHubContent structure for contents API
#[derive(Deserialize, Debug)]
struct GitHubContent {
    download_url: Option<String>,
    #[serde(rename = "type")]
    content_type: String,
    name: String,
}

// Add TreeResponse structures for the tree API
#[derive(Deserialize, Debug)]
struct TreeItem {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    item_type: String,
    sha: String,
    url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TreeResponse {
    tree: Vec<TreeItem>,
    truncated: bool,
}

// Rest of the original data structures
#[derive(Debug)]
pub struct EpochFolders {
    pub epoch_ending: String,
    pub state_epoch: String,
    pub transaction: String,
}

fn parse_state_epoch_version(folder_name: &str) -> Result<u64> {
    // Parse version from format: state_epoch_189_ver_64718615.0f7c
    let parts: Vec<&str> = folder_name.split("_ver_").collect();
    if parts.len() != 2 {
        bail!("Invalid state epoch folder format: {}", folder_name);
    }

    let version_str = parts[1].split('.').next().unwrap_or_default();
    u64::from_str(version_str)
        .with_context(|| format!("Failed to parse version from: {}", folder_name))
}

fn find_closest_transaction_folder(
    transaction_folders: &[(u64, String)],
    target_version: u64,
) -> Result<String> {
    // Find the highest version below target and lowest version above target
    let version_below = transaction_folders
        .iter()
        .filter(|(version, _)| version <= &target_version)
        .max_by_key(|(version, _)| version);

    let version_above = transaction_folders
        .iter()
        .filter(|(version, _)| version > &target_version)
        .min_by_key(|(version, _)| version);

    // Validate version ordering
    if let (Some((ver_below, _)), Some((ver_above, _))) = (version_below, version_above) {
        if ver_below >= ver_above {
            bail!(
                "Version ordering error: below ({}) >= above ({})",
                ver_below,
                ver_above
            );
        }
    }

    println!("For target version {}, found candidates:", target_version);
    if let Some((v, name)) = version_below {
        println!("  Below target: {} ({})", name, v);
    }
    if let Some((v, name)) = version_above {
        println!("  Above target: {} ({})", name, v);
    }

    // Choose the version below target
    version_below
        .map(|(_, name)| name.clone())
        .context("No suitable transaction folder found below target version")
}

pub async fn find_closest_epoch_folder(
    client: &Client,
    owner: &str,
    repo: &str,
    branch: &str,
    target_epoch: u64,
) -> Result<EpochFolders> {
    // Update to use the Git Tree API instead of Contents API
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}:snapshots",
        owner, repo, branch
    );

    let response = client
        .get(&api_url)
        .header("User-Agent", "libra-framework-downloader")
        .send()
        .await
        .context("Failed to list snapshots directory")?;

    let tree_response: TreeResponse = response
        .json()
        .await
        .context("Failed to parse snapshots directory contents")?;

    if tree_response.truncated {
        info!("Warning: GitHub Tree API response is truncated. Some folders might be missing.");
    }

    // Separate folders by type
    let mut epoch_ending_folders: Vec<(u64, String)> = Vec::new();
    let mut state_epoch_folders: Vec<(u64, String)> = Vec::new();
    let mut transaction_folders: Vec<(u64, String)> = Vec::new();

    for item in tree_response.tree {
        // Only consider tree items (directories)
        if item.item_type != "tree" {
            continue;
        }

        // Extract just the folder name from the path
        let folder_name = match Path::new(&item.path).file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        if let Some(epoch) = parse_epoch_ending_number(&folder_name) {
            epoch_ending_folders.push((epoch, folder_name));
        } else if let Some((epoch, _version)) = parse_state_epoch_info(&folder_name) {
            state_epoch_folders.push((epoch, folder_name));
        } else if let Some(version) = parse_transaction_number(&folder_name) {
            transaction_folders.push((version, folder_name));
        }
    }

    if epoch_ending_folders.is_empty()
        || state_epoch_folders.is_empty()
        || transaction_folders.is_empty()
    {
        bail!("Could not find all required folder types");
    }

    // Find closest matches for both folder types
    let closest_ending = find_closest_epoch_ending(&epoch_ending_folders, target_epoch)
        .context("Failed to find closest epoch_ending folder")?;

    let closest_state = find_matching_state_epoch(&state_epoch_folders, target_epoch)
        .context("Failed to find closest state_epoch folder")?;

    // Get version from state epoch folder
    let state_version = parse_state_epoch_version(&closest_state)?;

    // Find closest transaction folder
    let closest_transaction = find_closest_transaction_folder(&transaction_folders, state_version)
        .context("Failed to find suitable transaction folder")?;

    info!("Found closest folders for epoch {}:", target_epoch);
    info!("  epoch_ending: {}", closest_ending);
    info!("  state_epoch:  {}", closest_state);
    info!("  transaction:  {}", closest_transaction);

    Ok(EpochFolders {
        epoch_ending: closest_ending,
        state_epoch: closest_state,
        transaction: closest_transaction,
    })
}

fn find_closest_epoch_ending(folders: &[(u64, String)], target: u64) -> Result<String> {
    // For epoch_ending, find the highest epoch that's less than or equal to target
    folders
        .iter()
        .filter(|(epoch, _)| epoch <= &target)
        .max_by_key(|(epoch, _)| epoch)
        .map(|(_, name)| name.clone())
        .context("No suitable epoch_ending folder found")
}

fn find_matching_state_epoch(folders: &[(u64, String)], target: u64) -> Result<String> {
    // For state_epoch, we want an exact match if possible
    match folders
        .iter()
        .find(|(epoch, _)| epoch == &target)
        .map(|(_, name)| name.clone())
    {
        Some(name) => Ok(name),
        None => {
            // If no exact match, get the highest epoch that's less than target
            folders
                .iter()
                .filter(|(epoch, _)| epoch < &target)
                .max_by_key(|(epoch, _)| epoch)
                .map(|(_, name)| name.clone())
                .context("No suitable state_epoch folder found")
        }
    }
}

pub async fn download_github_folder(
    owner: &str,
    repo: &str,
    path: &str,
    branch: &str,
    output_dir: &str,
) -> Result<()> {
    let client = reqwest::Client::new();

    // Create the root output directory first
    fs::create_dir_all(output_dir)?;

    // Use the Git Tree API with recursive flag to get all contents at once
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}:{}?recursive=1",
        owner, repo, branch, path
    );

    info!("Downloading tree from: {}", api_url);

    let response = client
        .get(&api_url)
        .header("User-Agent", "libra-framework-downloader")
        .send()
        .await
        .context("Failed to send tree request")?;

    let tree_response: TreeResponse = response
        .json()
        .await
        .context("Failed to parse JSON tree response")?;

    if tree_response.truncated {
        info!("Warning: Response was truncated, not all files will be downloaded");
    }

    // Get the base path to properly handle nested directories
    let base_path = Path::new(path)
        .file_name()
        .map_or(String::new(), |name| name.to_string_lossy().to_string());
    let base_dir = Path::new(output_dir).join(base_path);
    fs::create_dir_all(&base_dir)?;

    // Process files from the tree
    for item in tree_response.tree {
        // Skip if not a blob (file)
        if item.item_type != "blob" {
            continue;
        }

        // The path in tree response is relative to the requested path
        let relative_path = item.path;

        // Compute where to save the file
        let output_path = base_dir.join(&relative_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Download the file content using raw GitHub URL
        let content_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}/{}",
            owner, repo, branch, path, relative_path
        );

        let content = client
            .get(&content_url)
            .header("User-Agent", "libra-framework-downloader")
            .send()
            .await
            .with_context(|| format!("Failed to download file: {}", relative_path))?
            .bytes()
            .await
            .with_context(|| format!("Failed to read bytes from: {}", relative_path))?;

        fs::write(&output_path, content)
            .with_context(|| format!("Failed to write file: {}", relative_path))?;
    }

    Ok(())
}

pub async fn download_restore_bundle(
    owner: &str,
    repo: &str,
    branch: &str,
    epoch_num: &u64,
    destination: &Path,
) -> Result<PathBuf> {
    // Create the bundle-specific directory
    let bundle_dir = destination.join(format!("epoch_{}_restore_bundle", epoch_num));
    if !bundle_dir.exists() {
        println!("creating directory: {}", bundle_dir.display());
        fs::create_dir_all(&bundle_dir)?;
    }

    let client = reqwest::Client::new();

    let folders = find_closest_epoch_folder(&client, owner, repo, branch, *epoch_num).await?;

    // Download all three folders
    for folder in [
        &folders.epoch_ending,
        &folders.state_epoch,
        &folders.transaction,
    ] {
        let snapshot_path = format!("snapshots/{}", folder);
        download_github_folder(
            owner,
            repo,
            &snapshot_path,
            branch,
            bundle_dir.to_str().unwrap(),
        )
        .await?;
    }

    println!(
        "Successfully downloaded restore bundle for epoch {} into {}",
        epoch_num,
        bundle_dir.display()
    );
    Ok(bundle_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_finding() {
        let epoch_endings = vec![
            (87, "epoch_ending_87-.0338".to_string()),
            (29, "epoch_ending_29-.abcd".to_string()),
            (15, "epoch_ending_15-.efgh".to_string()),
        ];

        let state_epochs = vec![
            (87, "state_epoch_87_ver_28871273.c7e6".to_string()),
            (29, "state_epoch_29_ver_12345678.abcd".to_string()),
            (15, "state_epoch_15_ver_87654321.efgh".to_string()),
        ];

        // Test exact matches
        assert_eq!(
            find_closest_epoch_ending(&epoch_endings, 29).unwrap(),
            "epoch_ending_29-.abcd"
        );
        assert_eq!(
            find_matching_state_epoch(&state_epochs, 29).unwrap(),
            "state_epoch_29_ver_12345678.abcd"
        );

        // Test closest matches
        assert_eq!(
            find_closest_epoch_ending(&epoch_endings, 30).unwrap(),
            "epoch_ending_29-.abcd"
        );
        assert_eq!(
            find_matching_state_epoch(&state_epochs, 30).unwrap(),
            "state_epoch_29_ver_12345678.abcd"
        );

        // Test no matches
        assert!(find_closest_epoch_ending(&epoch_endings, 10).is_err());
        assert!(find_matching_state_epoch(&state_epochs, 10).is_err());
    }

    #[test]
    fn test_transaction_folder_selection() {
        let folders = vec![
            (33100000, "transaction_33100000-.58b4".to_string()),
            (33000000, "transaction_33000000-.58b4".to_string()),
            (32900000, "transaction_32900000-.58b4".to_string()),
        ];

        // Should select 33000000 folder for version 33007311
        assert_eq!(
            find_closest_transaction_folder(&folders, 33007311).unwrap(),
            "transaction_33000000-.58b4"
        );
    }

    #[test]
    fn test_transaction_folder_version_ordering() {
        let folders = vec![
            (33100000, "transaction_33100000-.58b4".to_string()),
            (33000000, "transaction_33000000-.58b4".to_string()),
            (32900000, "transaction_32900000-.58b4".to_string()),
        ];

        // Test normal case
        assert_eq!(
            find_closest_transaction_folder(&folders, 33007311).unwrap(),
            "transaction_33000000-.58b4"
        );
    }
}
