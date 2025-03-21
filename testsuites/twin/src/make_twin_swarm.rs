use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use std::{path::PathBuf, time::Instant};

use libra_rescue::twin_setup;

use crate::twin_swarm::TwinSwarm;

/// Apply the rescue blob to the swarm db
/// returns the temp directory of the swarm
pub async fn make_twin_swarm(
    smoke: &mut LibraSmoke,
    reference_db: Option<PathBuf>,
    keep_running: bool,
) -> anyhow::Result<PathBuf> {
    let start_upgrade = Instant::now();

    // Collect credentials from all validators
    let creds = TwinSwarm::collect_validator_credentials(&smoke.swarm).await?;

    // Prepare the temporary database environment
    let (temp_db_path, _, start_version) =
        TwinSwarm::prepare_temp_database(&mut smoke.swarm, reference_db).await?;

    // Create and apply rescue blob
    let (rescue_blob_path, wp) = twin_setup::twin_e2e(&temp_db_path, creds).await?;

    // Update validators with the new DB and config
    // Self::update_nodes_with_rescue(&mut smoke.swarm, &temp_db_path, wp, rescue_blob_path).await?;

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

    if keep_running {
        dialoguer::Confirm::new()
            .with_prompt("swarm will keep running in background. Would you like to exit?")
            .interact()?;
        // NOTE: all validators will stop when the LibraSmoke goes out of context.
        // but since it's borrowed in this function you should assume it will continue until the caller goes out of scope.
    }

    Ok(temp_dir.to_owned())
}
