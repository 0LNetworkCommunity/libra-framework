use anyhow::Result;
use diem_config::config::NodeConfig;
use diem_forge::NodeExt;
use diem_forge::{LocalNode, LocalSwarm, SwarmExt};
use diem_temppath::TempPath;
use diem_types::waypoint::Waypoint;
use libra_config::validator_registration::ValCredentials;
use libra_smoke_tests::configure_validator;
use libra_smoke_tests::{
    extract_credentials::extract_swarm_node_credentials, libra_smoke::LibraSmoke,
};

use diem_config::config::InitialSafetyRulesConfig;
use diem_config::config::WaypointConfig;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::make_twin::{copy_dir_all, MakeTwin};
/// Manages swarm operations for a twin network
pub struct TwinSwarm;

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
    let (rescue_blob_path, wp) = MakeTwin::create_and_apply_rescue(&temp_db_path, creds).await?;

    // Update validators with the new DB and config
    TwinSwarm::update_nodes_with_rescue(&mut smoke.swarm, &temp_db_path, wp, rescue_blob_path)
        .await?;

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

impl TwinSwarm {
    /// Collect credentials from all validators in the swarm
    pub async fn collect_validator_credentials(swarm: &LocalSwarm) -> Result<Vec<ValCredentials>> {
        println!("Getting credentials from validator set");
        let mut creds = Vec::new();
        for n in swarm.validators() {
            let cred = extract_swarm_node_credentials(n).await?;
            creds.push(cred);
        }
        Ok(creds)
    }

    /// Replace DB for all validators in swarm
    pub async fn replace_db_all(swarm: &mut LocalSwarm, src_db_path: &Path) -> Result<()> {
        for n in swarm.validators_mut().into_iter() {
            let dst_db_path = n.config().storage.dir();
            copy_dir_all(src_db_path, &dst_db_path)?;
        }

        Ok(())
    }
    /// Apply the rescue blob to the swarm db
    fn update_node_config(validator: &mut LocalNode, mut config: NodeConfig) -> anyhow::Result<()> {
        // validator.stop();
        let node_path = validator.config_path();
        config.save_to_path(node_path)?;
        // validator.start()?;
        Ok(())
    }

    /// Update waypoint in all validator configs
    async fn update_waypoint(
        swarm: &mut LocalSwarm,
        wp: Waypoint,
        rescue_blob: PathBuf,
    ) -> anyhow::Result<()> {
        for n in swarm.validators_mut() {
            let mut node_config = n.config().clone();

            let configs_dir = &node_config.base.data_dir;

            let validator_identity_file = configs_dir.join("validator-identity.yaml");
            assert!(
                validator_identity_file.exists(),
                "validator-identity.yaml not found"
            );

            ////////
            // NOTE: you don't need to insert the waypoint as previously thought
            // but it is harmless. You must however set initial safety
            // rules config.
            // insert_waypoint(&mut node_config, wp);
            ///////

            let init_safety = InitialSafetyRulesConfig::from_file(
                validator_identity_file,
                WaypointConfig::FromConfig(wp),
            );
            node_config
                .consensus
                .safety_rules
                .initial_safety_rules_config = init_safety;

            ////////
            // Note: Example of getting genesis transaction serialized to include in config.
            // let genesis_transaction = {
            //     let buf = std::fs::read(rescue_blob.clone()).unwrap();
            //     bcs::from_bytes::<Transaction>(&buf).unwrap()
            // };
            /////////

            // NOTE: Must reset the genesis transaction in the config file
            // Or overwrite with a serialized versions
            node_config.execution.genesis = None; // see above to use bin: Some(genesis_transaction);
                                                  // ... and point to file
            node_config
                .execution
                .genesis_file_location
                .clone_from(&rescue_blob);

            Self::update_node_config(n, node_config)?;
        }
        Ok(())
    }
    /// Restart all validators
    pub fn restart_all(swarm: &mut LocalSwarm) -> Result<()> {
        for n in swarm.validators_mut().into_iter() {
            n.start()?;
        }

        Ok(())
    }

    /// Prepare the temporary database environment
    async fn prepare_temp_database(
        swarm: &mut LocalSwarm,
        reference_db: Option<PathBuf>,
    ) -> anyhow::Result<(PathBuf, PathBuf, u64)> {
        // Get starting version for verification
        let (start_version, _) = swarm
            .get_client_with_newest_ledger_version()
            .await
            .expect("could not get a client");

        // Stop all validators to prevent DB access conflicts
        for n in swarm.validators_mut() {
            n.stop();
        }

        // Use provided reference_db or first validator's DB
        let reference_db = reference_db
            .unwrap_or_else(|| swarm.validators().nth(0).unwrap().config().storage.dir());

        // Create temp directory for DB operations
        let mut temp = TempPath::new();
        temp.persist();
        temp.create_as_dir()?;
        let temp_path = temp.path();
        assert!(temp_path.exists());

        // Create a copy of the reference DB
        let temp_db_path = MakeTwin::temp_backup_db(&reference_db, temp_path)?;
        assert!(temp_db_path.exists());

        Ok((temp_db_path, reference_db, start_version))
    }

    /// Update nodes with rescue configuration
    pub async fn update_nodes_with_rescue(
        swarm: &mut LocalSwarm,
        temp_db_path: &Path,
        waypoint: Waypoint,
        rescue_blob_path: PathBuf,
    ) -> Result<()> {
        println!("Replacing swarm DB with the snapshot DB");
        Self::replace_db_all(swarm, temp_db_path).await?;

        println!("Updating waypoint in node configs and adding rescue blob");
        Self::update_waypoint(swarm, waypoint, rescue_blob_path).await?;

        Ok(())
    }

    /// Restart validators and verify successful operation
    pub async fn restart_and_verify(swarm: &mut LocalSwarm, start_version: u64) -> Result<()> {
        println!("Restarting validators and waiting for liveness");
        Self::restart_all(swarm)?;

        swarm
            .wait_for_all_nodes_to_catchup_to_version(start_version + 10, Duration::from_secs(30))
            .await?;

        Ok(())
    }
}
