use anyhow::Result;
use diem_config::config::NodeConfig;
use diem_types::waypoint::Waypoint;
use libra_config::make_yaml_public_fullnode::VALIDATOR_MANIFEST;
use libra_smoke_tests::{libra_smoke::LibraSmoke, smoke_test_environment::validator_swarm::LocalSwarm};
use libra_types::{validator_info::ValidatorInfo, twins::ValCredentials};
use std::{path::{Path, PathBuf}, time::{Duration, Instant}};

use crate::make_twin;

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

    /// Extract the credentials of the random validator
    async fn extract_swarm_node_credentials(marlon_node: &LocalNode) -> anyhow::Result<ValCredentials> {
        // get the necessary values from the current db
        let account = marlon_node.config().get_peer_id().unwrap();

        let public_identity_yaml = marlon_node
            .config_path()
            .parent()
            .unwrap()
            .join("public-identity.yaml");
        let public_identity =
            serde_yaml::from_slice::<PublicIdentity>(&fs::read(public_identity_yaml)?)?;
        let proof_of_possession = public_identity
            .consensus_proof_of_possession
            .unwrap()
            .to_bytes()
            .to_vec();
        let consensus_public_key_file = public_identity
            .consensus_public_key
            .clone()
            .unwrap()
            .to_string();

        // query the db for the values
        let query_res = query_view::get_view(
            &marlon_node.rest_client(),
            "0x1::stake::get_validator_config",
            None,
            Some(account.to_string()),
        )
        .await
        .unwrap();

        let network_addresses = query_res[1].as_str().unwrap().strip_prefix("0x").unwrap();
        let fullnode_addresses = query_res[2].as_str().unwrap().strip_prefix("0x").unwrap();
        let consensus_public_key_chain = query_res[0].as_str().unwrap().strip_prefix("0x").unwrap();

        // for checking if both values are the same:
        let consensus_public_key_chain = hex::decode(consensus_public_key_chain).unwrap();
        let consensus_pubkey = hex::decode(consensus_public_key_file).unwrap();
        let network_addresses = hex::decode(network_addresses).unwrap();
        let fullnode_addresses = hex::decode(fullnode_addresses).unwrap();

        assert_eq!(consensus_public_key_chain, consensus_pubkey);
        Ok(ValCredentials {
            account,
            consensus_pubkey,
            proof_of_possession,
            network_addresses,
            fullnode_addresses,
        })
    }
    /// Replace DB for all validators in swarm
    pub async fn replace_db_all(swarm: &mut LocalSwarm, src_db_path: &Path) -> Result<()> {
        for n in swarm.validators_mut().into_iter() {
            let dst_db_path = n.config().storage.dir();

            // Copy database files
            // This should be implemented based on your specific requirements
            // Example: fs::copy_dir_all(src_db_path, dst_db_path)?;
        }

        Ok(())
    }

    /// Update waypoint in all validator configs
    pub async fn update_waypoint(
        swarm: &mut LocalSwarm,
        waypoint: Waypoint,
        rescue_blob_path: PathBuf
    ) -> Result<()> {
        for n in swarm.validators_mut().into_iter() {
            let mut config = n.config().clone();

            // Update waypoint
            config.consensus.safety_rules.waypoint = Some(waypoint);

            // Update rescue blob path
            let mut safety_rules = &mut config.consensus.safety_rules;
            safety_rules.enable_rescue = true;
            safety_rules.rescue_path = Some(rescue_blob_path.clone());

            // Apply updated config
            n.restart_with_config(config)?;
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
    pub async fn restart_and_verify(
        swarm: &mut LocalSwarm,
        start_version: u64,
    ) -> Result<()> {
        println!("Restarting validators and waiting for liveness");
        Self::restart_all(swarm)?;

        swarm
            .wait_for_all_nodes_to_catchup_to_version(start_version + 10, Duration::from_secs(30))
            .await?;

        Ok(())
    }
}
