use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use std::{path::PathBuf, time::Instant};

use libra_rescue::{
    one_step::one_step_apply_rescue_on_db, replace_validators::replace_validators_blob,
};

use crate::twin_swarm::TwinSwarm;

/// Apply the rescue blob to the swarm db
/// returns the temp directory of the swarm
pub async fn awake_frankenswarm(
    smoke: &mut LibraSmoke,
    reference_db: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    let start_upgrade = Instant::now();

    // Collect credentials from all validators
    let creds = TwinSwarm::collect_validator_credentials(&smoke.swarm).await?;

    // Prepare the temporary database environment
    let (temp_db_path, _, start_version) =
        TwinSwarm::prepare_temp_database(&mut smoke.swarm, reference_db).await?;

    println!("Creating rescue blob from the reference db");
    let rescue_blob_path = replace_validators_blob(&temp_db_path, creds, &temp_db_path).await?;

    println!("Applying the rescue blob to the database & bootstrapping");
    let wp = one_step_apply_rescue_on_db(&temp_db_path, &rescue_blob_path)?;

    println!("4. Replace the swarm db with the snapshot db");
    TwinSwarm::replace_db_all(&mut smoke.swarm, &temp_db_path).await?;

    println!("5. Change the waypoint in the node configs and add the rescue blob to the config");
    TwinSwarm::update_waypoint(&mut smoke.swarm, wp, rescue_blob_path).await?;

    // Restart validators and verify operation
    TwinSwarm::restart_and_verify(&mut smoke.swarm, start_version).await?;

    // Generate CLI config files for validators
    configure_validator::save_cli_config_all(&mut smoke.swarm)?;

    let duration_upgrade = start_upgrade.elapsed();
    println!(
        "SUCCESS: twin swarm started. Time to prepare swarm: {:?}",
        duration_upgrade
    );

    let temp_dir = smoke.swarm.dir();
    println!("temp files found at: {}", temp_dir.display());

    Ok(temp_dir.to_owned())
}
