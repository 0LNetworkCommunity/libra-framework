use anyhow::Result;
use diem_temppath::TempPath;
use diem_types::waypoint::Waypoint;
use libra_config::validator_registration::ValCredentials;
use libra_rescue::{
    one_step::one_step_apply_rescue_on_db, replace_validators::replace_validators_blob,
};
// use libra_types::{
//     rescue::{build_rescue_network, save_rescue, RescueContext, RescueOptions},
//     twins::ValCredentials,
// };
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Handles database operations and rescue blob creation/application
pub struct MakeTwin;

impl MakeTwin {
    // /// Prepare a temporary database from a reference DB
    // pub fn prepare_temp_database(reference_db: &Path) -> Result<(PathBuf, PathBuf)> {
    //     // Create temp directory for DB operations
    //     let temp_dir = TempDir::new()?;
    //     let temp_path = temp_dir.path().to_path_buf();

    //     // Create a copy of the reference DB
    //     let temp_db_path = Self::temp_backup_db(reference_db, &temp_path)?;
    //     assert!(temp_db_path.exists());

    //     Ok((temp_db_path, temp_path))
    // }

    // /// Create a temporary backup of the database
    // pub fn temp_backup_db(from_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
    //     let mut tempdir = TempPath::new();
    //     tempdir.create_as_dir()?;
    //     tempdir.persist();

    //     let to_path = temp_dir.join("db_temp");

    //     // Ensure the destination directory exists
    //     fs::create_dir_all(&to_path)?;

    //     // Copy the database files
    //     fs::copy(from_path.join("consensus_db"), to_path.join("consensus_db"))?;

    //     fs::copy(
    //         from_path.join("secure_storage.json"),
    //         to_path.join("secure_storage.json"),
    //     )?;

    //     // Copy the ledger_db directory recursively
    //     let from_ledger = from_path.join("ledger_db");
    //     let to_ledger = to_path.join("ledger_db");
    //     fs::create_dir_all(&to_ledger)?;

    //     for entry in fs::read_dir(from_ledger)? {
    //         let entry = entry?;
    //         let path = entry.path();
    //         let to = to_ledger.join(path.file_name().unwrap());
    //         if path.is_dir() {
    //             fs::create_dir_all(&to)?;
    //             // Use fs_extra instead of custom function
    //             fs_extra::dir::copy(&path, &to, &CopyOptions::new())?;
    //         } else {
    //             fs::copy(&path, &to)?;
    //         }
    //     }

    //     Ok(to_path)
    // }

    // /// Create a rescue blob from credentials and a database
    // pub async fn make_rescue_twin_blob(
    //     temp_db_path: &Path,
    //     creds: Vec<ValCredentials>,
    // ) -> Result<PathBuf> {
    //     // Prepare the rescue context
    //     let mut rescue_options = RescueOptions::default();
    //     rescue_options.db_dir = Some(temp_db_path.to_path_buf());

    //     let rescue_context = RescueContext {
    //         namespace: "twinnet".to_string(),
    //         options: rescue_options,
    //         credentials: creds,
    //     };

    //     // Build the rescue network configuration
    //     let (vals_config, rescue_data, _) = build_rescue_network(&rescue_context)
    //         .await
    //         .context("Could not build rescue blob")?;

    //     // Save the rescue blob to a file
    //     let f = NamedTempFile::new()?;
    //     let rescue_path = f.path().to_path_buf();
    //     save_rescue(&rescue_path, rescue_data)?;

    //     Ok(rescue_path)
    // }

    // /// Apply a rescue blob to a database
    // pub fn apply_rescue_on_db(db_path: &Path, rescue_path: &Path) -> Result<Waypoint> {
    //     // Open the database
    //     let db_rw = DbReaderWriter::new(diem_db::DiemDB::open(db_path, false, None, false, None)?);

    //     // Read and parse the rescue blob
    //     let blob_bytes = fs::read(rescue_path)?;
    //     let rescue: Transaction = bcs::from_bytes(&blob_bytes)?;

    //     // Generate a waypoint from the transaction
    //     let waypoint = generate_waypoint::<DiemVM>(&db_rw, &rescue)?;

    //     // Bootstrap the database
    //     // This will execute the rescue transaction
    //     diem_executor::db_bootstrapper::maybe_bootstrap::<DiemVM>(&db_rw, &rescue, waypoint)?;

    //     Ok(waypoint)
    // }

    /// Create and apply rescue blob in one operation
    pub async fn create_and_apply_rescue(
        temp_db_path: &Path,
        creds: Vec<ValCredentials>,
    ) -> Result<(PathBuf, Waypoint)> {
        println!("Creating rescue blob from the reference db");
        let rescue_blob_path = replace_validators_blob(temp_db_path, creds).await?;

        println!("Applying the rescue blob to the database & bootstrapping");
        let waypoint = one_step_apply_rescue_on_db(temp_db_path, &rescue_blob_path)?;

        Ok((rescue_blob_path, waypoint))
    }
}
