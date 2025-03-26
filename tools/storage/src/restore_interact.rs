use anyhow::Result;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use diem_logger::info;
use fs_extra::dir;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use crate::storage_cli::{StorageCli, Sub};
use std::{path::PathBuf, time::Duration};

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
pub async fn restore_db(
    bundle_path: PathBuf,
    db_path: PathBuf,
    epoch: Option<u64>,
    owner: Option<String>,
    repo: Option<String>,
    branch: Option<String>,
) -> Result<()> {
    // If epoch is provided, download the bundle first
    if let Some(epoch_num) = epoch {
        let owner_str = owner.unwrap_or_else(|| "0LNetworkCommunity".to_string());
        let repo_str = repo.unwrap_or_else(|| "epoch-archive-mainnet".to_string());
        let branch_str = branch.unwrap_or_else(|| "v7.0.0".to_string());

        info!(
            "Starting download process for epoch {} to {}",
            epoch_num,
            bundle_path.display()
        );

        let download_cli = StorageCli {
            command: Some(Sub::DownloadRestoreBundle {
                owner: owner_str,
                repo: repo_str,
                branch: branch_str,
                epoch: epoch_num,
                destination: bundle_path.clone(),
            }),
        };
        download_cli.run().await?;
    } else {
        info!("Using existing bundle at {}", bundle_path.display());
    }

    // Restore and bootstrap the DB using the EpochRestore subcommand
    info!("Restoring and bootstrapping DB to {}", db_path.display());
    let restore_cli = StorageCli {
        command: Some(Sub::EpochRestore {
            bundle_path,
            destination_db: db_path,
            prevent_bootstrap: false, // We want to bootstrap by default
        }),
    };
    restore_cli.run().await?;

    info!("Restore process completed successfully");
    Ok(())
}

/// Interactive dialogue to guide users through the restore process
/// Asks users a series of questions and then performs the restore operation
/// Uses dialoguer and indicatif for a better user experience
///
/// # Arguments
/// * `epoch_arg` - Optional epoch number to restore (skips prompting if provided)
///
/// # Returns
/// * `Result<()>` - Ok if the operation is successful
pub async fn interactive_restore(epoch_arg: Option<u64>) -> Result<()> {
    // Set up progress indicators
    let multi_progress = MultiProgress::new();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{spinner} {msg}")
        .unwrap();


    // Ask if user wants to download a restore point
    let download = if epoch_arg.is_some() {
        // If epoch is provided, assume we're downloading
        true
    } else {
        Confirm::new()
            .with_prompt("Do you want to download a restore point?")
            .default(true)
            .interact()?
    };

    // Get bundle path
    let default_bundle_path = PathBuf::from(".")
        .join("restore_bundle");

    let prompt = format!("Where do you want to {} the restore bundle?",
        if download { "download" } else { "find" });

    let bundle_path: PathBuf = Input::<String>::new()
        .with_prompt(&prompt)
        .default(*default_bundle_path.to_str().unwrap())
        .interact()?;

    // Get DB path
    let default_db_path = PathBuf::from(".")
        .join("db");

    let db_path: PathBuf = Input::new()
        .with_prompt("Where do you want to restore the DB?")
        .default(default_db_path)
        .interact()?;

    // If downloading, get epoch and other params
    let mut epoch = epoch_arg;
    let mut owner = None;
    let mut repo = None;
    let mut branch = None;

    if download {
        // Only ask for epoch if not provided as an argument
        if epoch.is_none() {
            let epoch_input: u64 = Input::new()
                .with_prompt("What epoch do you want to restore?")
                .interact()?;

            epoch = Some(epoch_input);
        }

        // Advanced options
        let advanced = Confirm::new()
            .with_prompt("Do you want to set advanced options?")
            .default(false)
            .interact()?;

        if advanced {
            let owner_input: String = Input::new()
                .with_prompt("GitHub organization/user")
                .default("0LNetworkCommunity".to_string())
                .interact()?;

            if !owner_input.is_empty() {
                owner = Some(owner_input);
            }

            let repo_input: String = Input::new()
                .with_prompt("Repository name")
                .default("epoch-archive-mainnet".to_string())
                .interact()?;

            if !repo_input.is_empty() {
                repo = Some(repo_input);
            }

            let branch_input: String = Input::new()
                .with_prompt("Branch name")
                .default("v7.0.0".to_string())
                .interact()?;

            if !branch_input.is_empty() {
                branch = Some(branch_input);
            }
        }
    }

    // Confirm operation
    println!("\nSummary of restore operation:");
    if download {
        println!("- Download restore bundle for epoch {} to {}",
            epoch.unwrap(), bundle_path.display());
    } else {
        println!("- Use existing restore bundle at {}", bundle_path.display());
    }
    println!("- Restore DB to {}", db_path.display());

    let confirm = Confirm::new()
        .with_prompt("\nProceed with operation?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("Operation cancelled by user.");
        return Ok(());
    }

    // Create directories if they don't exist
    if !bundle_path.exists() {
        std::fs::create_dir_all(&bundle_path)?;
    }
    if !db_path.exists() {
        std::fs::create_dir_all(&db_path)?;
    }

    println!();

    // Progress spinners for operations
    let download_spinner = if download {
        let pb = multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(spinner_style.clone());
        pb.set_message(format!("Downloading restore bundle for epoch {}...", epoch.unwrap()));
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let restore_spinner = multi_progress.add(ProgressBar::new_spinner());
    restore_spinner.set_style(spinner_style);
    restore_spinner.set_message("Waiting to start restore process...");
    restore_spinner.enable_steady_tick(Duration::from_millis(100));

    // Call restore_db with collected parameters
    if let Some(epoch_num) = epoch {
        let owner_str = owner.unwrap_or_else(|| "0LNetworkCommunity".to_string());
        let repo_str = repo.unwrap_or_else(|| "epoch-archive-mainnet".to_string());
        let branch_str = branch.unwrap_or_else(|| "v7.0.0".to_string());

        let download_cli = StorageCli {
            command: Some(Sub::DownloadRestoreBundle {
                owner: owner_str,
                repo: repo_str,
                branch: branch_str,
                epoch: epoch_num,
                destination: bundle_path.clone(),
            }),
        };

        download_cli.run().await?;

        if let Some(spinner) = &download_spinner {
            spinner.finish_with_message(format!("Download completed for epoch {}", epoch_num));
        }
    }

    // Restore the DB
    restore_spinner.set_message(format!("Restoring DB to {}...", db_path.display()));

    let restore_cli = StorageCli {
        command: Some(Sub::EpochRestore {
            bundle_path,
            destination_db: db_path,
            prevent_bootstrap: false, // We want to bootstrap by default
        }),
    };

    restore_cli.run().await?;
    restore_spinner.finish_with_message("Database restore completed successfully");

    println!("\n✅ Restore operation completed successfully!");
    println!("You can now start your node using the restored DB.");

    Ok(())
}
