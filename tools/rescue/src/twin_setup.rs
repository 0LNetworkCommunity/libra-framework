use anyhow::Result;
use libra_config::validator_registration::ValCredentials;
use std::path::{Path, PathBuf};

use diem_types::waypoint::Waypoint;

use crate::{one_step::one_step_apply_rescue_on_db, replace_validators::replace_validators_blob};

pub async fn twin_e2e(
    reference_db: &Path,
    creds: Vec<ValCredentials>,
) -> Result<(PathBuf, Waypoint)> {
    println!("Creating rescue blob from the reference db");
    let rescue_blob_path = replace_validators_blob(reference_db, creds).await?;

    println!("Applying the rescue blob to the database & bootstrapping");
    let waypoint = one_step_apply_rescue_on_db(reference_db, &rescue_blob_path)?;

    Ok((rescue_blob_path, waypoint))
}
