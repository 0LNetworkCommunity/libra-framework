#![allow(unused)]
use crate::diem_db_bootstrapper::BootstrapOpts;
use crate::session_tools::session_add_validators;
use anyhow::bail;
use anyhow::Context;
use clap::Parser;
use diem_config::config::WaypointConfig;
use diem_types::transaction::Script;
use libra_config::make_profile;
use libra_smoke_tests::helpers::mint_libra;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::txs_cli_vals::ValidatorTxs;
use std::process::abort;
use std::time::Instant;
use std::{path::PathBuf, time::Duration};

use smoke_test::test_utils::{
    swarm_utils::insert_waypoint, MAX_CATCH_UP_WAIT_SECS, MAX_CONNECTIVITY_WAIT_SECS,
    MAX_HEALTHY_WAIT_SECS,
};

use diem_config::config::NodeConfig;
use diem_forge::Swarm;
use diem_forge::SwarmExt;
use diem_forge::Validator;
use diem_temppath::TempPath;
use diem_types::transaction::{Transaction, TransactionPayload, WriteSetPayload};
use diem_types::validator_config::ValidatorOperatorConfigResource;
use fs_extra::dir;
use futures_util::TryFutureExt;
use libra_smoke_tests::configure_validator;
use libra_smoke_tests::helpers;
use libra_smoke_tests::helpers::get_libra_balance;
use move_core_types::account_address::AccountAddress;
use tokio::process::Command;

use libra_txs::txs_cli::{TxsCli, TxsSub::Transfer};
use libra_types::legacy_types::app_cfg::TxCost;

use crate::{rescue_tx::RescueTxOpts, session_tools::ValCredentials};
use diem_api_types::ViewRequest;
use diem_config::config::InitialSafetyRulesConfig;
use diem_config::keys::ConfigKey;
use diem_crypto::bls12381;
use diem_crypto::bls12381::ProofOfPossession;
use diem_crypto::ed25519::PrivateKey;
use diem_forge::{LocalNode, LocalVersion, Node, NodeExt, Version};
use diem_genesis::config::HostAndPort;
use diem_genesis::keys::{PrivateIdentity, PublicIdentity};
use diem_types::on_chain_config::new_epoch_event_key;
use diem_types::waypoint::Waypoint;
use diem_vm::move_vm_ext::SessionExt;
use hex;
use hex::FromHex;
use libra_config::validator_config;
use libra_query::query_view;
use libra_types::exports::{Client, NamedChain};
use libra_wallet::core::legacy_scheme::LegacyKeyScheme;
use libra_wallet::validator_files::SetValidatorConfiguration;
use move_core_types::value::MoveValue;
use serde::Deserialize;
use std::fs;
use std::mem::ManuallyDrop;
use std::path::Path;

use crate::session_tools::{
    self, libra_execute_session_function, libra_run_session, writeset_voodoo_events,
};

#[derive(Parser)]

/// set up a twin of the network, with a synced db
/// and replace the validator with local Swarm validators
pub struct TwinOpts {
    // path of snapshot db we want marlon to drive
    #[clap(value_parser)]
    pub db_dir: PathBuf,
    /// The operator.yaml file which contains registration information
    #[clap(value_parser)]
    pub oper_file: Option<PathBuf>,
    /// provide info about the DB state, e.g. version
    #[clap(value_parser)]
    pub info: bool,
}

impl TwinOpts {
    /// takes a snapshot db and makes a validator set of ONE from an
    /// existing or NEW marlon rando account
    pub fn run(&self) -> anyhow::Result<()> {
        if self.info {
            return Ok(());
        }

        Ok(())
    }

    /// we need a new account config created locally
    async fn initialize_marlon_the_val() -> anyhow::Result<PathBuf> {
        // we use LibraSwarm to create a new folder with validator configs.
        // we then take the operator.yaml, and use it to register on a dirty db
        let mut s = LibraSmoke::new(Some(1), None).await?;
        s.swarm.wait_all_alive(Duration::from_secs(10)).await?;
        let marlon = s.swarm.validators_mut().next().unwrap();
        marlon.stop();

        Ok(marlon.config_path().join("operator.yaml"))
    }

    /// create the validator registration entry function payload
    /// needs the file operator.yaml
    fn register_marlon_tx(file: PathBuf) -> anyhow::Result<Script> {
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

    /// create the rescue blob which has one validator
    fn recue_blob_with_one_val() {}

    pub async fn make_rescue_twin_blob(
        db_path: &Path,
        creds: Vec<&ValCredentials>,
    ) -> anyhow::Result<PathBuf> {
        println!("run session to create validator onboarding tx (rescue.blob)");
        let epoch_interval = 100_u64;
        let vmc = libra_run_session(
            db_path.to_path_buf(),
            |session| session_add_validators(session, creds),
            None,
            Some(epoch_interval),
        )?;

        let cs = session_tools::unpack_changeset(vmc)?;

        let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));
        let out = db_path.join("rescue.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&out, bytes.as_slice())?;
        Ok(out)
    }

    pub fn update_node_config_restart(
        validator: &mut LocalNode,
        mut config: NodeConfig,
    ) -> anyhow::Result<()> {
        validator.stop();
        let node_path = validator.config_path();
        config.save_to_path(node_path)?;
        validator.start()?;
        Ok(())
    }

    /// end to end with rando
    /// Which is basically running a new random swarm on an existing db.
    pub async fn apply_with_rando_e2e(
        prod_db: PathBuf,
    ) -> anyhow::Result<LibraSmoke, anyhow::Error> {
        //The diem-node should be compiled externally to avoid any potential conflicts with the current build
        //get the current path
        let current_path = std::env::current_dir()?;
        //path to diem-node binary
        let diem_node_path = current_path.join("tests/diem-proxy");
        //number of nodes to create
        let number_of_nodes = 3;
        // 1. Create a new validator set with new accounts
        println!("1. Create a new validator set with new accounts");
        // Or provide a path to the diem-node binary
        std::env::set_var("DIEM_FORGE_NODE_BIN_PATH", "/root/.cargo/diem-node");
        let mut smoke = LibraSmoke::new(Some(number_of_nodes), None).await?;
        std::env::remove_var("DIEM_FORGE_NODE_BIN_PATH");

        //due to borrowing issues
        let client = smoke.client().clone();

        //Get the credentials of all the nodes
        let mut creds = Vec::new();
        for n in smoke.swarm.validators() {
            let cred = Self::extract_credentials(n).await?;
            creds.push(cred);
        }

        //convert from Vec<ValCredentials> to Vec<&ValCredentials>
        let creds = creds.iter().collect::<Vec<_>>();

        // 2.Replace the swarm db with the brick db
        println!("2.Replace the swarm db with the brick db");
        let swarm_db_paths = smoke
            .swarm
            .validators()
            .map(|n| n.config().storage.dir())
            .collect::<Vec<_>>();

        smoke.swarm.validators_mut().for_each(|n| {
            n.stop();
            n.clear_storage();
        });
        swarm_db_paths.iter().for_each(|p| {
            Self::clone_db(&prod_db, p).unwrap();
        });

        swarm_db_paths.iter().for_each(|p| {
            assert!(p.exists());
        });

        let first_val = smoke.swarm.validators().next().unwrap().peer_id();
        let genesis_blob_path =
            Self::make_rescue_twin_blob(&swarm_db_paths[0], creds.to_owned()).await?;
        let mut genesis_blob_paths = Vec::new();
        genesis_blob_paths.push(genesis_blob_path.clone());
        // 3. Apply the rescue blob to the swarm db
        println!("3. Apply the rescue blob to the swarm db");
        for (i, p) in swarm_db_paths.iter().enumerate() {
            //copy the genesis blob to the other swarm nodes dbachives directories
            if i == 0 {
                continue;
            }
            let out = p.join("rescue.blob");
            std::fs::copy(&genesis_blob_path, &out)?;
            genesis_blob_paths.push(out.to_owned());
        }

        let mut waypoints = Vec::new();
        //Apply the rescue blob to the swarm db
        println!("4. Bootstrap the swarm db with the rescue blob");
        for (i, p) in swarm_db_paths.iter().enumerate() {
            let bootstrap = BootstrapOpts {
                db_dir: p.clone(),
                genesis_txn_file: genesis_blob_paths[i].clone(),
                waypoint_to_verify: None,
                commit: false, // NOT APPLYING THE TX
                info: false,
            };

            let waypoint = bootstrap.run()?;
            dbg!(&waypoint);

            //give time for any IO to finish
            std::thread::sleep(Duration::from_secs(10));

            let bootstrap = BootstrapOpts {
                db_dir: p.clone(),
                genesis_txn_file: genesis_blob_paths[i].clone(),
                waypoint_to_verify: None,
                commit: true, // APPLY THE TX
                info: false,
            };

            let waypoint = bootstrap.run().unwrap().unwrap();

            waypoints.push(waypoint);
        }

        // Chenge the waypoint in the node configs and add the rescue blob to the config
        println!(
            "
            5. Chenge the waypoint in the node configs and add the rescue blob to the config"
        );
        for (i, n) in smoke.swarm.validators_mut().enumerate() {
            let mut config = n.config().clone();
            let mut node_config = n.config().clone();
            insert_waypoint(&mut node_config, waypoints[i]);
            node_config
                .consensus
                .safety_rules
                .initial_safety_rules_config = InitialSafetyRulesConfig::FromFile {
                identity_blob_path: genesis_blob_paths[i].clone(),
                waypoint: WaypointConfig::FromConfig(waypoints[i]),
            };
            let genesis_transaction = {
                let buf = std::fs::read(&genesis_blob_paths[i].clone()).unwrap();
                bcs::from_bytes::<Transaction>(&buf).unwrap()
            };
            node_config.execution.genesis = Some(genesis_transaction);
            // reset the sync_only flag to false
            node_config.consensus.sync_only = false;
            Self::update_node_config_restart(n, node_config)?;
            Self::wait_for_node(n, i).await?;
        }
        println!("6. wait for liveness");
        smoke
            .swarm
            .liveness_check(Instant::now().checked_add(Duration::from_secs(10)).unwrap());
        //DO NOT FORGET TO CHANGE THE MAX GAS UNIT ALLOWED
        //path to config file
        //peer id of all the validators
        let peer_ids = smoke
            .swarm
            .validators()
            .map(|n| n.config().validator_network.as_ref().unwrap().peer_id())
            .collect::<Vec<_>>();
        println!("peer ids of validators: {:?}", peer_ids);

        let a = client
            .view(
                &ViewRequest {
                    function: "0x1::stake::get_current_validators".parse().unwrap(),
                    type_arguments: vec![],
                    arguments: vec![],
                },
                None,
            )
            .await;
        println!("Current validators: {:?}", a);
        // TO DO: REVESIT THIS TRANSACTION
        /*
        let d = diem_temppath::TempPath::new();
        let (_, _app_cfg) =
            configure_validator::init_val_config_files(&mut smoke.swarm, 0, d.path().to_owned())
                .await
                .expect("could not init validator config");
        let recipient = smoke.swarm.validators().nth(1).unwrap().peer_id(); // sending to second genesis node.
        let marlon = smoke.swarm.validators().nth(0).unwrap().peer_id();
        let bal = get_libra_balance(&client, marlon).await?;
        println!("balance of marlon: {:?}", bal);
        let cli = TxsCli {
            subcommand: Some(Transfer {
                to_account: recipient,
                amount: 1.0,
            }),
            mnemonic: None,
            test_private_key: Some(smoke.encoded_pri_key.clone()),
            chain_id: None,
            config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
            url: Some(smoke.api_endpoint.clone()),
            tx_profile: None,
            tx_cost: Some(TxCost::default_baseline_cost()),
            estimate_only: false,
        };
        cli.run()
            .await
            .expect("cli could not send to existing account");
        std::thread::sleep(Duration::from_secs(1000));
        */
        std::thread::sleep(Duration::from_secs(1000));
        Ok(smoke)
    }

    /// from an initialized swarm state, extract one node's credentials
    async fn extract_credentials(marlon_node: &LocalNode) -> anyhow::Result<ValCredentials> {
        println!("extracting swarm validator credentpials");
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

    fn clone_db(prod_db: &Path, swarm_db: &Path) -> anyhow::Result<()> {
        println!("copying the db db to the swarm db");
        println!("prod db path: {:?}", prod_db);
        println!("swarm db path: {:?}", swarm_db);

        // this swaps the directories
        assert!(prod_db.exists());
        assert!(swarm_db.exists());
        let swarm_old_path = swarm_db.parent().unwrap().join("db-old");
        fs::create_dir(&swarm_old_path);
        let options = dir::CopyOptions::new(); //Initialize default values for CopyOptions

        // move source/dir1 to target/dir1
        dir::move_dir(swarm_db, &swarm_old_path, &options)?;
        assert!(!swarm_db.exists());

        fs::create_dir(swarm_db);
        dir::copy(prod_db, swarm_db.parent().unwrap(), &options)?;

        println!("db copied");
        Ok(())
    }

    pub async fn wait_for_node(
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

#[ignore]
#[tokio::test]
// cargo test test_twin_with_rando -- --nocapture
async fn test_twin_with_rando() -> anyhow::Result<()> {
    //use any db
    let prod_db_to_clone = PathBuf::from("/root/.libra/db");
    TwinOpts::apply_with_rando_e2e(prod_db_to_clone).await?;
    println!("Test passed");
    Ok(())
}
