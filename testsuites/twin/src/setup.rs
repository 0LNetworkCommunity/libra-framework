use anyhow::{bail, Context};
use diem_config::config::{NodeConfig, WaypointConfig};
use diem_forge::{LocalSwarm, SwarmExt, Validator};
use diem_temppath::TempPath;
use diem_types::{
    transaction::{Script, Transaction, WriteSetPayload},
    waypoint::Waypoint,
};
use fs_extra::dir;
use libra_smoke_tests::{configure_validator, helpers::get_libra_balance, libra_smoke::LibraSmoke};
use libra_txs::txs_cli_vals::ValidatorTxs;
use smoke_test::test_utils::{
    swarm_utils::insert_waypoint, MAX_CONNECTIVITY_WAIT_SECS, MAX_HEALTHY_WAIT_SECS,
};
use std::{
    path::PathBuf,
    process::abort,
    time::{Duration, Instant},
};

use libra_config::validator_registration::ValCredentials;
use libra_txs::txs_cli::{TxsCli, TxsSub::Transfer};
use libra_types::core_types::app_cfg::TxCost;

use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::{LocalNode, Node, NodeExt};
use diem_genesis::keys::PublicIdentity;
use hex::{self};
use libra_query::query_view;
use libra_rescue::{
    diem_db_bootstrapper::BootstrapOpts,
    session_tools::{self, libra_run_session, session_add_validators},
};
use std::{fs, path::Path};

use crate::runner::Twin;

/// Setup the twin network with a synced db
impl Twin {
    /// initialize swarm and return the operator.yaml (keys) from
    /// the first validator (marlon rando)
    pub async fn initialize_marlon_the_val() -> anyhow::Result<PathBuf> {
        // we use LibraSwarm to create a new folder with validator configs.
        // we then take the operator.yaml, and use it to register on a dirty db
        let mut s = LibraSmoke::new(Some(1), None).await?;
        s.swarm.wait_all_alive(Duration::from_secs(10)).await?;
        let marlon = s.swarm.validators_mut().next().unwrap();
        marlon.stop();

        Ok(marlon.config_path().join("operator.yaml"))
    }
    // TODO: do we need this?
    /// create the validator registration entry function payload
    /// needs the file operator.yaml
    pub fn register_marlon_tx(file: PathBuf) -> anyhow::Result<Script> {
        let tx = ValidatorTxs::Register {
            operator_file: Some(file),
        }
        .make_payload()?
        .encode();
        if let diem_types::transaction::TransactionPayload::Script(s) = tx {
            return Ok(s);
        }
        bail!("function did not return a script")
    }
    ///  Make a rescue blob with the given credentials
    async fn make_rescue_twin_blob(
        db_path: &Path,
        creds: Vec<ValCredentials>,
    ) -> anyhow::Result<PathBuf> {
        println!("run session to create validator onboarding tx (rescue.blob)");
        let vmc = libra_run_session(
            db_path.to_path_buf(),
            |session| session_add_validators(session, creds, false),
            None,
            None,
        )?;

        let cs = session_tools::unpack_changeset(vmc)?;

        let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));
        let out = db_path.join("rescue.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&out, bytes.as_slice())?;
        Ok(out)
    }

    /// Apply the rescue blob to the swarm db
    fn update_node_config_restart(
        validator: &mut LocalNode,
        mut config: NodeConfig,
    ) -> anyhow::Result<()> {
        validator.stop();
        let node_path = validator.config_path();
        config.save_to_path(node_path)?;
        validator.start()?;
        Ok(())
    }

    async fn replace_db(swarm: &mut LocalSwarm, reference_db: PathBuf) -> anyhow::Result<()> {
        for v in swarm.validators_mut() {
            // stop and clear storage
            v.stop();
            v.clear_storage().await?;

            // clone the DB
            Self::clone_db(&reference_db, &v.config().storage.dir())?;
        }
        Ok(())
    }

    fn rescue_db_with_blob(swarm: &LocalSwarm, rescue_blob: PathBuf) -> anyhow::Result<Waypoint> {
        let mut waypoint: Option<Waypoint> = None;
        for v in swarm.validators() {
            let storage_dir = v.config().storage.dir();
            let bootstrap = BootstrapOpts {
                db_dir: storage_dir.clone(),
                genesis_txn_file: rescue_blob.clone(),
                waypoint_to_verify: None,
                commit: false, // NOT APPLYING THE TX
                info: false,
            };

            let waypoint_to_check = bootstrap.run()?.expect("could not get waypoint");
            dbg!(&waypoint_to_check);

            // give time for any IO to finish
            std::thread::sleep(Duration::from_secs(10));

            let bootstrap = BootstrapOpts {
                db_dir: storage_dir.clone(),
                genesis_txn_file: rescue_blob.clone(),
                waypoint_to_verify: Some(waypoint_to_check),
                commit: true, // APPLY THE TX
                info: false,
            };

            let waypoint_post = bootstrap.run()?.expect("could not get waypoint");
            assert!(
                waypoint_to_check == waypoint_post,
                "waypoints are not equal"
            );
            if let Some(w) = waypoint {
                assert!(
                    waypoint_to_check == w,
                    "waypoints are not equal between nodes"
                );
            }
            waypoint = Some(waypoint_to_check);
        }
        match waypoint {
            Some(w) => Ok(w),
            None => bail!("cannot generate consistent waypoint."),
        }
    }

    async fn update_waypoint(
        swarm: &mut LocalSwarm,
        wp: Waypoint,
        rescue_blob: PathBuf,
    ) -> anyhow::Result<()> {
        for (i, n) in swarm.validators_mut().enumerate() {
            let mut node_config = n.config().clone();
            insert_waypoint(&mut node_config, wp.clone());

            // TODO:
            node_config
                .consensus
                .safety_rules
                .initial_safety_rules_config = InitialSafetyRulesConfig::None;
            // };
            // let genesis_transaction = {
            //     let buf = std::fs::read(rescue_blob.clone()).unwrap();
            //     bcs::from_bytes::<Transaction>(&buf).unwrap()
            // };
            node_config.execution.genesis_file_location = rescue_blob.clone();
            // reset the sync_only flag to false
            node_config.consensus.sync_only = false;
            Self::update_node_config_restart(n, node_config)?;
            Self::wait_for_node(n, i).await?;
        }
        Ok(())
    }

    /// Apply the rescue blob to the swarm db
    pub async fn apply_with_rando_e2e(
        reference_db: PathBuf,
        num_validators: u8,
    ) -> anyhow::Result<(LibraSmoke, PathBuf), anyhow::Error> {
        let start_upgrade = Instant::now();

        println!("1. Create a new validator set with new accounts");
        let mut smoke = LibraSmoke::new(Some(num_validators), None).await?;

        //Get the credentials of all the nodes
        let mut creds = Vec::new();
        for n in smoke.swarm.validators() {
            let cred = Self::extract_credentials(n).await?;
            creds.push(cred);
        }

        let creds = creds.into_iter().collect::<Vec<_>>();

        println!("2.Replace the swarm db with the snapshot db");
        Self::replace_db(&mut smoke.swarm, reference_db).await?;

        println!("3. Create a rescue blob with the new validator");
        let one_storage_dir = smoke
            .swarm
            .validators()
            .nth(0)
            .unwrap()
            .config()
            .storage
            .dir();

        let rescue_blob_path = Self::make_rescue_twin_blob(&one_storage_dir, creds).await?;

        println!("4. Apply the rescue blob to the swarm db");
        println!("5. Bootstrap the swarm db with the rescue blob");

        let wp = Self::rescue_db_with_blob(&smoke.swarm, rescue_blob_path.clone())?;

        println!(
            "6. Change the waypoint in the node configs and add the rescue blob to the config"
        );
        Self::update_waypoint(&mut smoke.swarm, wp, rescue_blob_path).await?;

        println!("7. wait for liveness");
        // TODO: check if this is doing what is expected
        // was previously not running
        smoke
            .swarm
            .liveness_check(Instant::now().checked_add(Duration::from_secs(20)).unwrap())
            .await?;

        let cli_tools = smoke.first_account_app_cfg()?;

        let duration_upgrade = start_upgrade.elapsed();
        println!(">>> Time to prepare swarm: {:?}", duration_upgrade);

        Ok((smoke, cli_tools.workspace.node_home))
    }

    /// Extract the credentials of the random validator
    async fn extract_credentials(marlon_node: &LocalNode) -> anyhow::Result<ValCredentials> {
        println!("extracting swarm validator credentials");
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

    /// Clone the prod db to the swarm db
    fn clone_db(prod_db: &Path, swarm_db: &Path) -> anyhow::Result<()> {
        println!("copying the db db to the swarm db");
        println!("prod db path: {:?}", prod_db);
        println!("swarm db path: {:?}", swarm_db);

        // this swaps the directories
        assert!(prod_db.exists());
        assert!(swarm_db.exists());
        let swarm_old_path = swarm_db.parent().unwrap().join("db-old");
        fs::create_dir(&swarm_old_path)?;
        let options = dir::CopyOptions::new(); // Initialize default values for CopyOptions

        // move source/dir1 to target/dir1
        dir::move_dir(swarm_db, &swarm_old_path, &options)?;
        assert!(!swarm_db.exists());

        fs::create_dir(swarm_db)?;
        dir::copy(prod_db, swarm_db.parent().unwrap(), &options)?;

        println!("db copied");
        Ok(())
    }

    /// Wait for the node to become healthy
    async fn wait_for_node(
        validator: &mut dyn Validator,
        expected_to_connect: usize,
    ) -> anyhow::Result<()> {
        let healthy_deadline = Instant::now()
            .checked_add(Duration::from_secs(MAX_HEALTHY_WAIT_SECS))
            .context("no deadline")?;
        validator
            .wait_until_healthy(healthy_deadline)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Error waiting for node to become healthy: {}", e);
                abort();
            });

        let connectivity_deadline = Instant::now()
            .checked_add(Duration::from_secs(MAX_CONNECTIVITY_WAIT_SECS))
            .context("can't get new deadline")?;
        validator
            .wait_for_connectivity(expected_to_connect, connectivity_deadline)
            .await?;
        Ok(())
    }
}
