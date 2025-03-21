use crate::parse_folder_names::{
    parse_epoch_ending_number, parse_state_epoch_info, parse_transaction_number,
};
use anyhow::bail;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
struct GitHubContent {
    download_url: Option<String>,
    #[serde(rename = "type")]
    content_type: String,
    name: String,
}

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
    dbg!(&contents);
    // Separate folders by type
    let mut epoch_ending_folders: Vec<(u64, String)> = Vec::new();
    let mut state_epoch_folders: Vec<(u64, String)> = Vec::new();
    let mut transaction_folders: Vec<(u64, String)> = Vec::new();

    for item in contents {
        if item.content_type != "dir" {
            continue;
        }

        if let Some(epoch) = parse_epoch_ending_number(&item.name) {
            epoch_ending_folders.push((epoch, item.name));
        } else if let Some((epoch, _version)) = parse_state_epoch_info(&item.name) {
            state_epoch_folders.push((epoch, item.name));
        } else if let Some(version) = parse_transaction_number(&item.name) {
            transaction_folders.push((version, item.name));
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

    println!("Found closest folders for epoch {}:", target_epoch);
    println!("  epoch_ending: {}", closest_ending);
    println!("  state_epoch:  {}", closest_state);
    println!("  transaction:  {}", closest_transaction);

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
    let mut pending_dirs = vec![(path.to_string(), output_dir.to_string())];

    // Create the root output directory first
    fs::create_dir_all(output_dir)?;

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

        // Extract the last component of the path to create the current directory
        let current_folder = Path::new(&current_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Create full directory path including the current folder
        let full_dir_path = Path::new(&current_dir).join(&current_folder);
        fs::create_dir_all(&full_dir_path)?;

        for item in contents {
            let output_path = full_dir_path.join(&item.name);

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
                pending_dirs.push((new_path, full_dir_path.to_str().unwrap().to_string()));
            }
        }
    }

    Ok(())
}

pub async fn download_restore_bundle(
    owner: &str,
    repo: &str,
    branch: &str,
    epoch_num: &u64,
    destination: &Path,
) -> Result<()> {
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
    Ok(())
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

        // Test error case with invalid ordering
        let invalid_folders = vec![
            (33000000, "transaction_33000000-.58b4".to_string()),
            (33000000, "transaction_33000000-.58b4".to_string()), // Duplicate version
        ];
        assert!(find_closest_transaction_folder(&invalid_folders, 33007311).is_err());
    }
}
