use anyhow::{bail, Context};
use diem_config::config::{NodeConfig, WaypointConfig};
use diem_forge::{LocalSwarm, SwarmExt, Validator};
use diem_temppath::TempPath;
use diem_types::{
    transaction::{Script, Transaction, WriteSetPayload},
    waypoint::Waypoint,
};
use fs_extra::dir;
use libra_smoke_tests::libra_smoke::LibraSmoke;
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

use diem_config::config::InitialSafetyRulesConfig;
use diem_forge::{LocalNode, Node, NodeExt};
use diem_genesis::keys::PublicIdentity;
use hex::{self};
use libra_query::query_view;
use libra_rescue::{
    diem_db_bootstrapper::BootstrapOpts,
    session_tools::{self, libra_run_session, session_add_validators, writeset_voodoo_events},
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
            // |s| writeset_voodoo_events(s),
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
    fn update_node_config(validator: &mut LocalNode, mut config: NodeConfig) -> anyhow::Result<()> {
        // validator.stop();
        let node_path = validator.config_path();
        config.save_to_path(node_path)?;
        // validator.start()?;
        Ok(())
    }

    /// Apply the rescue blob to the swarm db
    fn restart_all(swarm: &mut LocalSwarm) -> anyhow::Result<()> {
        for v in swarm.validators_mut() {
            v.start()?;
        }
        Ok(())
    }

    async fn replace_db_all(swarm: &mut LocalSwarm, reference_db: &Path) -> anyhow::Result<()> {
        for v in swarm.validators_mut() {
            // stop and clear storage
            v.stop();
            v.clear_storage().await?;

            // clone the DB
            Self::clone_db(reference_db, &v.config().storage.dir())?;
        }
        Ok(())
    }

    fn temp_backup_db(reference_db: &Path, temp_dir: &Path) -> anyhow::Result<PathBuf> {
        let options: dir::CopyOptions = dir::CopyOptions::new(); // Initialize default values for CopyOptions
        dir::copy(reference_db, temp_dir, &options).context("cannot copy to new db dir")?;
        let db_path = temp_dir.join("db");
        assert!(db_path.exists());

        Ok(db_path)
    }

    fn apply_rescue_on_db(
        db_to_change_path: &Path,
        rescue_blob: &Path,
    ) -> anyhow::Result<Waypoint> {
        let mut waypoint: Option<Waypoint> = None;
        let bootstrap = BootstrapOpts {
            db_dir: db_to_change_path.to_owned(),
            genesis_txn_file: rescue_blob.to_owned(),
            waypoint_to_verify: None,
            commit: false, // NOT APPLYING THE TX
            info: false,
        };

        let waypoint_to_check = bootstrap.run()?.expect("could not get waypoint");

        // give time for any IO to finish
        std::thread::sleep(Duration::from_secs(1));

        let bootstrap = BootstrapOpts {
            db_dir: db_to_change_path.to_owned(),
            genesis_txn_file: rescue_blob.to_owned(),
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
        // }
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
        for n in swarm.validators_mut() {
            let mut node_config = n.config().clone();

            // DON'T INSERT WAYPOINT
            // insert_waypoint(&mut node_config, wp);

            let configs_dir = &node_config.base.data_dir;

            let validator_identity_file = configs_dir.join("validator-identity.yaml");
            assert!(validator_identity_file.exists(), "validator-identity.yaml not found");

            let init_safety = InitialSafetyRulesConfig::from_file(
                validator_identity_file,
                WaypointConfig::FromConfig(wp),
            );
            node_config
                .consensus
                .safety_rules
                .initial_safety_rules_config = init_safety;

            let genesis_transaction = {
                let buf = std::fs::read(rescue_blob.clone()).unwrap();
                bcs::from_bytes::<Transaction>(&buf).unwrap()
            };
            node_config
            .execution
            .genesis = Some(genesis_transaction);

            // node_config
            //     .execution
            //     .genesis_file_location
            //     .clone_from(&rescue_blob);

            Self::update_node_config(n, node_config)?;

        }
        Ok(())
    }

    /// Apply the rescue blob to the swarm db
    pub async fn make_twin_swarm(
        smoke: &mut LibraSmoke,
        reference_db: Option<PathBuf>,
        keep_running: bool,
    ) -> anyhow::Result<PathBuf> {
        let start_upgrade = Instant::now();

        println!("1. Get credentials from validator set.");

        //Get the credentials of all the nodes
        let mut creds = Vec::new();
        for n in smoke.swarm.validators() {
            let cred = Self::extract_credentials(n).await?;
            creds.push(cred);
        }

        // stop all vals so we don't have DBs open.
        let (start_version, _) = smoke
            .swarm
            .get_client_with_newest_ledger_version()
            .await
            .expect("could not get a client");
        for n in smoke.swarm.validators_mut() {
            n.stop();
        }

        let creds = creds.into_iter().collect::<Vec<_>>();

        // If no DB is sent, we will use the swarm's initial DB,
        // this is useful for debugging the internals of Twin, since
        // we should expect no changes to validator set, credentials and state, only the rescue transaction.
        let reference_db = reference_db.unwrap_or_else(|| {
            smoke
                .swarm
                .validators()
                .nth(0)
                .unwrap()
                .config()
                .storage
                .dir()
        });

        // Do all writeset operations on a temp db.
        let mut temp = TempPath::new();
        temp.persist();
        temp.create_as_dir()?;
        let temp_path = temp.path();
        assert!(temp_path.exists());
        let temp_db_path = Self::temp_backup_db(&reference_db, temp_path)?;
        dbg!(&temp_db_path);
        assert!(temp_db_path.exists());

        println!("2. Create a rescue blob from the reference db");

        let rescue_blob_path = Self::make_rescue_twin_blob(&temp_db_path, creds).await?;


        println!("3. Apply the rescue blob to the swarm db & bootstrap");

        let wp = Self::apply_rescue_on_db(&temp_db_path, &rescue_blob_path)?;

        println!("4. Replace the swarm db with the snapshot db");

        Self::replace_db_all(&mut smoke.swarm, &temp_db_path).await?;


        println!(
            "5. Change the waypoint in the node configs and add the rescue blob to the config"
        );
        Self::update_waypoint(&mut smoke.swarm, wp, rescue_blob_path).await?;

        println!("6. wait for liveness");
        Self::restart_all(&mut smoke.swarm)?;

        // TODO: check if this is doing what is expected
        // was previously not running
        // smoke
        //     .swarm
        //     .liveness_check(Instant::now().checked_add(Duration::from_secs(20)).unwrap())
        //     .await?;

        smoke
            .swarm
            .wait_for_all_nodes_to_catchup_to_version(start_version + 10, Duration::from_secs(30))
            .await?;

        let cli_tools = smoke.first_account_app_cfg()?;

        let duration_upgrade = start_upgrade.elapsed();
        println!(
            "SUCCESS: twin swarm started. Time to prepare swarm: {:?}",
            duration_upgrade
        );

        if keep_running {
            dialoguer::Confirm::new()
                .with_prompt("swarm will keep running in background. Would you like to exit?")
                .interact()?;
        }
        // NOTE: all validators will stop when the LibraSmoke goes out of context.
        Ok(cli_tools.workspace.node_home)
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

    /// Wait for the node to become healthy
    async fn _wait_for_node(
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

#[tokio::test]
async fn test_setup_twin_with_noop_db() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(Some(1), None).await?;

    let version_old = smoke.client().get_ledger_information().await?.inner().version;
    Twin::make_twin_swarm(&mut smoke, None, false).await?;

    let version_now = smoke.client().get_ledger_information().await?.inner().version;

    assert!(version_now > version_old, "chain makes progress");

    Ok(())
}
