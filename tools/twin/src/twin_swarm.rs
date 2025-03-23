use anyhow::{Context, Result};
use diem_config::config::NodeConfig;
use diem_forge::{LocalNode, LocalSwarm, SwarmExt};
use diem_temppath::TempPath;
use diem_types::waypoint::Waypoint;
use libra_config::validator_registration::ValCredentials;
use libra_smoke_tests::extract_credentials::extract_swarm_node_credentials;

use diem_config::config::InitialSafetyRulesConfig;
use diem_config::config::WaypointConfig;
use fs_extra::dir::{self, CopyOptions};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use diem_forge::Node;
/// Manages swarm operations for a twin network
pub struct TwinSwarm;

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
    pub async fn replace_db_on_swarm_nodes(
        swarm: &mut LocalSwarm,
        src_db_path: &Path,
    ) -> Result<()> {
        dbg!(&src_db_path);
        for n in swarm.validators_mut() {
            let dst_db_path = n.config().storage.dir();
            fs_extra::dir::copy(
                src_db_path,
                &dst_db_path,
                &CopyOptions::new().content_only(true).overwrite(true),
            )?;
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
    /// Prepare the temporary database environment
    pub async fn prepare_temp_database(
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
        let temp_db_path = Self::temp_backup_db(&reference_db, temp_path)?;
        assert!(temp_db_path.exists());

        Ok((temp_db_path, reference_db, start_version))
    }

    fn temp_backup_db(reference_db: &Path, temp_dir: &Path) -> anyhow::Result<PathBuf> {
        let options: dir::CopyOptions = dir::CopyOptions::new(); // Initialize default values for CopyOptions
        dir::copy(reference_db, temp_dir, &options).context("cannot copy to new db dir")?;
        let db_path = temp_dir.join(reference_db.file_name().unwrap().to_str().unwrap());
        assert!(db_path.exists());

        Ok(db_path)
    }

    pub async fn replace_db_all(swarm: &mut LocalSwarm, reference_db: &Path) -> anyhow::Result<()> {
        for v in swarm.validators_mut() {
            // stop and clear storage
            v.stop();
            v.clear_storage().await?;

            // clone the DB
            clone_db(reference_db, &v.config().storage.dir())?;
        }
        Ok(())
    }
    /// Update waypoint in all validator configs
    pub async fn update_waypoint(
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
        for n in swarm.validators_mut() {
            n.start()?;
        }

        Ok(())
    }

    /// Update nodes with rescue configuration
    pub async fn update_nodes_with_rescue(
        swarm: &mut LocalSwarm,
        temp_db_path: &Path,
        waypoint: Waypoint,
        rescue_blob_path: PathBuf,
    ) -> Result<()> {
        println!("Replacing swarm DB with the snapshot DB");
        Self::replace_db_on_swarm_nodes(swarm, temp_db_path).await?;

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

/// Clone the prod db to the swarm db
fn clone_db(prod_db: &Path, swarm_db: &Path) -> anyhow::Result<()> {
    println!("copying the db db to the swarm db");
    println!("prod db path: {:?}", prod_db);
    println!("swarm db path: {:?}", swarm_db);

    // this swaps the directories
    assert!(prod_db.exists());
    assert!(swarm_db.exists());
    let swarm_old_path = swarm_db.parent().unwrap().join("db-old");
    match fs::create_dir(&swarm_old_path).context("cannot create db-old") {
        Ok(_) => {}
        Err(e) => {
            println!(
                "db-old path already exists at {:?}, {}",
                &swarm_old_path,
                &e.to_string()
            );
        }
    };
    let options = dir::CopyOptions::new(); // Initialize default values for CopyOptions

    // move source/dir1 to target/dir1
    dir::move_dir(swarm_db, &swarm_old_path, &options)?;
    assert!(
        !swarm_db.exists(),
        "swarm db should have been moved/deleted"
    );

    fs::create_dir(swarm_db).context("cannot create new db dir")?;

    dir::copy(prod_db, swarm_db.parent().unwrap(), &options)
        .context("cannot copy to new db dir")?;

    println!("db copied");
    Ok(())
}
