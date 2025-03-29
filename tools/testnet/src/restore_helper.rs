use anyhow::Result;
use libra_storage::{download_bundle, restore};
use std::path::PathBuf;

/// Helper to do a single step operation to download a backup archive from epoch archive,
/// confirm the files are downloaded, then create a new db from the backup.
/// The DB will then be ready for the twin writeset.
///
/// This function can either:
/// 1. Download a bundle and then restore it (if `epoch` is provided)
/// 2. Restore from an existing bundle (if `epoch` is None)
///
/// # Arguments
/// * `bundle_path` - Path to the bundle (existing or to be downloaded)
/// * `db_path` - Path where the new DB should be created
/// * `epoch` - Optional epoch number to restore (when downloading)
/// * `owner` - Optional GitHub organization or user (default: "0LNetworkCommunity")
/// * `repo` - Optional repository name (default: "epoch-archive-mainnet")
/// * `branch` - Optional branch name (default: "v7.0.0")
///
/// # Returns
/// * `Result<()>` - Ok if the operation is successful
pub async fn one_step_restore_db(
    data_path: PathBuf,
    epoch: u64,
    owner: Option<String>,
    repo: Option<String>,
    branch: Option<String>,
) -> Result<PathBuf> {
    println!(
        "restoring twin db for epoch {} at {}",
        epoch,
        data_path.display()
    );

    let owner_str = owner.unwrap_or_else(|| "0LNetworkCommunity".to_string());
    let repo_str = repo.unwrap_or_else(|| "epoch-archive-mainnet".to_string());
    let branch_str = branch.unwrap_or_else(|| "v7.0.0".to_string());

    println!(
        "Starting download process for epoch {} to {}",
        epoch,
        data_path.display()
    );

    let restore_path = download_bundle::download_restore_bundle(
        &owner_str,
        &repo_str,
        &branch_str,
        &epoch,
        &data_path,
    )
    .await?;

    let destination_db = data_path.join(format!("db_{epoch}"));
    // Restore and bootstrap the DB using the EpochRestore subcommand
    println!(
        "Restoring and bootstrapping DB to {}",
        destination_db.display()
    );
    restore::epoch_restore(restore_path, destination_db.clone()).await?;

    println!("Restore process completed successfully");
    Ok(destination_db)
}
