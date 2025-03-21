use diem_types::waypoint::Waypoint;
use serde::Deserialize;
use std::path::PathBuf;
use anyhow::Result;

pub const GENESIS_FILES_VERSION: &str = "7.0.0";

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct GithubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
}

/// download genesis blob and save it to the specified directory.
pub async fn download_genesis(home_dir: Option<PathBuf>) -> Result<()> {
    // Base URL for GitHub API requests
    let base_url =
        "https://api.github.com/repos/0LNetworkCommunity/epoch-archive-mainnet/contents/upgrades";
    let client = reqwest::Client::new();
    let resp = client
        .get(base_url)
        .header("User-Agent", "request")
        .send()
        .await?
        .json::<Vec<GithubContent>>()
        .await?;

    // Find the latest version by parsing version numbers and sorting
    let latest_version = resp
        .iter()
        .filter_map(|entry| entry.name.split('v').nth(1)) // Assuming the name is 'vX.X.X'
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let latest_path = format!(
        "{}/v{}/genesis.blob",
        "https://raw.githubusercontent.com/0LNetworkCommunity/epoch-archive-mainnet/main/upgrades",
        latest_version.unwrap_or(GENESIS_FILES_VERSION)
    );

    // Fetch the latest waypoint
    let blob_bytes = reqwest::get(&latest_path).await?.bytes().await?;
    let home = home_dir.unwrap_or_else(libra_types::global_config_dir);
    let genesis_dir = home.join("genesis/");

    // Ensure the genesis directory exists
    std::fs::create_dir_all(&genesis_dir)?;

    let p = genesis_dir.join("genesis.blob");

    // Write the genesis blob to the file
    std::fs::write(p, &blob_bytes)?;
    Ok(())
}

/// Fetch the genesis waypoint from the GitHub repository.
pub async fn get_genesis_waypoint(home_dir: Option<PathBuf>) -> Result<Waypoint> {
    // Base URL for GitHub API requests
    let base_url =
        "https://api.github.com/repos/0LNetworkCommunity/epoch-archive-mainnet/contents/upgrades";
    let client = reqwest::Client::new();

    // Fetch the list of upgrade versions
    let resp = client
        .get(base_url)
        .header("User-Agent", "request")
        .send()
        .await?
        .json::<Vec<GithubContent>>()
        .await?;

    // Find the latest version by parsing version numbers and sorting
    let latest_version = resp
        .iter()
        .filter_map(|entry| entry.name.split('v').nth(1)) // Assuming the name is 'vX.X.X'
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let latest_path = format!(
        "{}/v{}/waypoint.txt",
        "https://raw.githubusercontent.com/0LNetworkCommunity/epoch-archive-mainnet/main/upgrades",
        latest_version.unwrap_or(GENESIS_FILES_VERSION)
    );

    // Fetch the latest waypoint
    let wp_string = reqwest::get(&latest_path).await?.text().await?;
    let home = home_dir.unwrap_or_else(libra_types::global_config_dir);
    let genesis_dir = home.join("genesis/");
    let p = genesis_dir.join("waypoint.txt");

    // Save the waypoint to a file
    std::fs::write(p, &wp_string)?;
    wp_string.trim().parse::<Waypoint>()
}

#[tokio::test]
async fn persist_genesis() {
    let mut p = diem_temppath::TempPath::new();
    p.create_as_dir().unwrap();
    p.persist();

    let path = p.path().to_owned();

    // Ensure the directory exists
    assert!(
        std::fs::metadata(&path).is_ok(),
        "Directory does not exist: {:?}",
        path
    );

    // Verify path is a directory
    assert!(
        Path::new(&path).is_dir(),
        "Path is not a directory: {:?}",
        path
    );

    // Attempt to download genesis
    download_genesis(Some(path.clone())).await.unwrap();

    // Check if the genesis.blob file exists
    let genesis_blob_path = path.join("genesis").join("genesis.blob");
    assert!(
        genesis_blob_path.exists(),
        "genesis.blob file does not exist: {:?}",
        genesis_blob_path
    );
}
