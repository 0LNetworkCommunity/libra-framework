use crate::find_operators::find_operator_configs;
use crate::replace_validators_file::replace_validators_blob;
use libra_config::validator_registration::ValCredentials;
use libra_rescue::{
    cli_bootstrapper::one_step_apply_rescue_on_db, node_config::post_rescue_node_file_updates,
};
use std::path::{Path, PathBuf};

/// Configure a twin network based on the specified options
pub async fn configure_twin(
    home_path: &Path,
    reference_db: &Path,
    upgrade_mrb_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    // don't do any operations on the reference db

    assert!(home_path.exists(), "home data path should exist");
    // Note this is the customary path for the database
    let destination_db = home_path.join("data/db");
    std::fs::create_dir_all(&destination_db)?;

    println!("Copying reference db to: {}", destination_db.display());

    fs_extra::dir::copy(
        reference_db,
        &destination_db, // will create a folder under this path
        &fs_extra::dir::CopyOptions::new()
            .content_only(true)
            .overwrite(true),
    )?;

    assert!(destination_db.exists(), "destination db should exist");
    // Step 1: Collect all the operator.yaml files
    println!("Collecting operator configuration files...");

    let val_credentials: Vec<ValCredentials> = find_operator_configs(home_path)?;
    // Step 2 & 3: Run the twin rescue mission with the database path
    println!("Running twin rescue mission...");
    // Create and apply rescue blob
    println!("Creating rescue blob from the reference db");
    let rescue_blob_path =
        replace_validators_blob(reference_db, val_credentials, home_path, upgrade_mrb_path).await?;
    println!("Created rescue blob at: {}", rescue_blob_path.display());

    println!("Applying the rescue blob to the database & bootstrapping");
    let wp = one_step_apply_rescue_on_db(reference_db, &rescue_blob_path)?;
    println!("Writeset successful, waypoint: {}", wp);

    // Step 4: Update config files with artifacts
    println!("Updating configuration files...");
    let config_path = home_path.join("validator.yaml");

    post_rescue_node_file_updates(&config_path, wp, &rescue_blob_path)?;

    println!("Twin configuration complete");
    Ok(())
}
