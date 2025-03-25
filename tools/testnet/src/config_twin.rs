use crate::replace_validators_file::replace_validators_blob;

use glob::glob;
use libra_config::validator_registration::{registration_from_operator_yaml, ValCredentials};
use libra_rescue::cli_bootstrapper::one_step_apply_rescue_on_db;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::twin_swarm::update_genesis_in_node_config;

/// Configure a twin network based on the specified options
pub async fn configure_twin(home_path: &Path, reference_db: &Path) -> anyhow::Result<()> {
    // don't do any operations on the reference db
    let destination_db = home_path.join("data/db");
    fs::create_dir_all(&destination_db).await?;
    fs_extra::dir::copy(
        reference_db,
        &destination_db, // saving to standard db path
        &fs_extra::dir::CopyOptions::new(),
    )?;
    assert!(destination_db.exists(), "destination db should exist");
    println!("Copied reference db to: {}", destination_db.display());
    // Step 1: Collect all the operator.yaml files
    println!("Collecting operator configuration files...");
    // using glob read all the operator*.yaml files in <data_path>/operator_files
    let operator_path = home_path.join("operator_files");
    let pattern = operator_path
        .join("operator*.yaml")
        .to_string_lossy()
        .to_string();

    let mut operator_files: Vec<PathBuf> = Vec::new();
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("Found operator file: {:?}", path);
                operator_files.push(path);
            }
            Err(e) => println!("Error while processing operator file: {}", e),
        }
    }

    if operator_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No operator files found in {}",
            operator_path.display()
        ));
    }

    // Parse each operator file into ValCredentials
    let mut val_credentials: Vec<ValCredentials> = Vec::new();
    for path in operator_files {
        match registration_from_operator_yaml(Some(path)) {
            Ok(cred) => {
                println!(
                    "Successfully parsed credentials for account: {}",
                    cred.account
                );
                val_credentials.push(cred);
            }
            // You know a man of my ability
            // He should be smokin' on a big cigar
            // But 'til I get myself straight I guess I'll just have to wait
            // In my rubber suit rubbin' these cars
            // Well, all I can do is to shake my head
            // You might not believe that it's true
            // For workin' at this end of Niagara Falls
            // Is an undiscovered Howard Hughes
            // So baby, don't expect to see me
            // With no double martini in any high brow society news
            // 'Cause I got them steadily depressin', low down mind messin'
            // Workin' at the car wash blues
            // So baby, don't expect to see me
            // With no double martini in any high brow society news
            // 'Cause I got them steadily depressin', low down mind messin'
            // Workin' at the car wash blues
            // Yeah, I got them steadily depressin', low down mind messin'
            // Workin' at the car wash blues
            Err(e) => println!("Error parsing operator file: {}", e),
        }
    }

    if val_credentials.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid credentials could be parsed from operator files"
        ));
    }

    // Step 2 & 3: Run the twin rescue mission with the database path
    println!("Running twin rescue mission...");
    // Create and apply rescue blob
    println!("Creating rescue blob from the reference db");
    let rescue_blob_path =
        replace_validators_blob(reference_db, val_credentials, home_path).await?;
    println!("Created rescue blob at: {}", rescue_blob_path.display());

    println!("Applying the rescue blob to the database & bootstrapping");
    let wp = one_step_apply_rescue_on_db(reference_db, &rescue_blob_path)?;
    println!("Writeset successful, waypoint: {}", wp);

    // Step 4: Update config files with artifacts
    println!("Updating configuration files...");
    let config_path = home_path.join("validator.yaml");
    update_genesis_in_node_config(&config_path, &rescue_blob_path, wp)?;

    println!("Twin configuration complete");
    Ok(())
}
