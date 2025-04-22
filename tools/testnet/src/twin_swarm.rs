use crate::{cli_output::TestInfo, replace_validators_file::replace_validators_blob};
use anyhow::{Context, Result};
use diem_forge::{LocalSwarm, SwarmExt};
use diem_temppath::TempPath;
use diem_types::chain_id::NamedChain;
use diem_types::waypoint::Waypoint;
use fs_extra::dir;
use libra_config::validator_registration::ValCredentials;
use libra_rescue::cli_bootstrapper::one_step_apply_rescue_on_db;
use libra_smoke_tests::extract_credentials::extract_swarm_node_credentials;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_types::core_types::app_cfg::AppCfg;
use std::time::Instant;
use std::{
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

    pub async fn replace_db_all(swarm: &mut LocalSwarm, patched_db: &Path) -> anyhow::Result<()> {
        for v in swarm.validators_mut() {
            // stop and clear storage
            v.stop();
            v.clear_storage().await?;

            // clone the DB
            dir::copy(
                patched_db,
                v.config().storage.dir(),
                &dir::CopyOptions::new().content_only(true).overwrite(true),
            )
            .context("cannot copy to new db dir")?;
        }
        Ok(())
    }

    /// Update waypoint in all validator configs
    pub async fn update_node_files(
        swarm: &mut LocalSwarm,
        wp: Waypoint,
        rescue_blob: PathBuf,
    ) -> anyhow::Result<()> {
        for n in swarm.validators_mut() {
            libra_rescue::node_config::post_rescue_node_file_updates(
                &n.config_path(),
                wp,
                &rescue_blob,
            )?;
        }
        Ok(())
    }

    /// update the AppCfg in each validator data path,
    /// to use the twin chain id i.e, TESTNET (2) (not! TESTING(4))
    pub async fn update_app_cfg(swarm: &LocalSwarm) -> anyhow::Result<()> {
        for n in swarm.validators() {
            let cfg = n.config_path();
            let data_path = cfg.parent().unwrap();
            let mut app_cfg = AppCfg::load(Some(data_path.join("libra-cli-config.yaml")))?;
            let np = app_cfg.get_network_profile_mut(None)?;
            np.chain_name = NamedChain::TESTNET;
            app_cfg.workspace.default_chain_id = NamedChain::TESTNET;

            app_cfg.save_file()?;
        }
        Ok(())
    }

    /// Restart validators and verify successful operation
    pub async fn restart_and_verify(swarm: &mut LocalSwarm, start_version: u64) -> Result<()> {
        println!("Restarting validators and waiting for liveness");
        for n in swarm.validators_mut() {
            n.start()?;
        }

        swarm
            .wait_for_all_nodes_to_catchup_to_version(start_version + 10, Duration::from_secs(30))
            .await?;

        Ok(())
    }
}

/// Apply the rescue blob to the swarm db
/// returns the temp directory of the swarm
pub async fn awake_frankenswarm(
    smoke: &mut LibraSmoke,
    reference_db: Option<PathBuf>,
    framework_mrb_path: Option<PathBuf>,
) -> anyhow::Result<Vec<TestInfo>> {
    let start_upgrade = Instant::now();

    // Collect credentials from all validators
    let creds = TwinSwarm::collect_validator_credentials(&smoke.swarm).await?;

    let (start_version, _client) = smoke
        .swarm
        .get_client_with_newest_ledger_version()
        .await
        .expect("could not get node status");

    // Stop all validators to prevent DB access conflicts

    for n in smoke.swarm.validators_mut() {
        n.stop();
    }

    // temp db path (separate from swarm temp path)
    // we'll do operations on the temp db path not the actual reference
    let temp = TempPath::new();
    temp.create_as_dir()?;
    let temp_db_path = temp.path();

    // use the provided reference_db or get the one from the virgin swarm
    let reference_db = reference_db.unwrap_or(
        smoke
            .swarm
            .validators()
            .next()
            .unwrap()
            .config()
            .storage
            .dir(),
    );

    dir::copy(
        reference_db,
        temp_db_path,
        &dir::CopyOptions::new().content_only(true).overwrite(true),
    )
    .context("cannot copy to new db dir")?;

    println!("Creating rescue blob from the reference db");
    let rescue_blob_path =
        replace_validators_blob(temp_db_path, creds, temp_db_path, framework_mrb_path).await?;

    println!("Applying the rescue blob to the database & bootstrapping");
    let wp = one_step_apply_rescue_on_db(temp_db_path, &rescue_blob_path)?;

    println!("Replace the swarm db with the snapshot db");
    TwinSwarm::replace_db_all(&mut smoke.swarm, temp_db_path).await?;

    println!("Change the waypoint in the node configs and add the rescue blob to the config");
    TwinSwarm::update_node_files(&mut smoke.swarm, wp, rescue_blob_path).await?;
    TwinSwarm::update_app_cfg(&mut smoke.swarm).await?;

    // Restart validators and verify operation
    TwinSwarm::restart_and_verify(&mut smoke.swarm, start_version).await?;

    let duration_upgrade = start_upgrade.elapsed();
    println!(
        "SUCCESS: twin smoke started. Time to prepare: {:?}",
        duration_upgrade
    );

    let test_info = TestInfo::from_smoke(&smoke.swarm)?;

    Ok(test_info)
}
