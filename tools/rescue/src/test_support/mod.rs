// TODO: Move this and other generally useful
// test fixtures to its own module.
#![allow(dead_code)]
use anyhow::Context;
use diem_config::config::NodeConfig;
use diem_forge::{LocalNode, NodeExt, Validator};
use diem_logger::info;
use diem_temppath::TempPath;
use diem_types::account_address::AccountAddress;
use libra_framework::framework_cli::make_template_files;
use libra_types::core_types::app_cfg::TxCost;
use smoke_test::test_utils::{MAX_CONNECTIVITY_WAIT_SECS, MAX_HEALTHY_WAIT_SECS};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use diem_types::chain_id::NamedChain;
use flate2::read::GzDecoder;
use libra_framework::upgrade_fixtures;
use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Governance},
    txs_cli_governance::GovernanceTxs::{Propose, Resolve, Vote},
};
use std::path::Path;
use tar::Archive;

/// Sets up a test database by extracting a fixture file to a temporary directory.
///
/// This function extracts the database fixture from `./rescue/fixtures/db_339.tar.gz`,
/// which contains a recovered database at epoch 339. The extracted database is placed
/// in a temporary directory that will be automatically cleaned up when the TempPath
/// is dropped (unless persist() is called).
///
/// Returns the PathBuf containing the extracted database.
pub fn setup_v7_reference_twin_db() -> anyhow::Result<PathBuf> {
    let mut temp_dir = TempPath::new();
    temp_dir.create_as_dir()?;
    temp_dir.persist();

    // Open and decompress the fixture file
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixture_path = Path::new(manifest_dir).join("fixtures/db_339.tar.gz");
    assert!(&fixture_path.exists(), "can't find fixture db_339.tar.gz");
    let tar_gz = fs::File::open(fixture_path)?;
    let decompressor = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressor);

    // Extract to temp directory
    archive.unpack(temp_dir.path())?;

    Ok(temp_dir.path().join("db_339"))
}

pub fn make_script(remove_validator: AccountAddress) -> PathBuf {
    let script = format!(
        r#"
        script {{
            use diem_framework::stake;
            use diem_framework::diem_governance;
            use diem_framework::block;

            fun main(vm_signer: &signer, framework_signer: &signer) {{
                stake::remove_validators(framework_signer, &vector[@0x{:?}]);
                block::emit_writeset_block_event(vm_signer, @0x1);
                diem_governance::reconfigure(framework_signer);
            }}
    }}
    "#,
        remove_validator
    );
    println!("{}", script);
    let framework_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("framework")
        .join("libra-framework");

    let mut temp_script_path = TempPath::new();
    temp_script_path.create_as_dir().unwrap();
    temp_script_path.persist();

    assert!(temp_script_path.path().exists());

    make_template_files(
        temp_script_path.path(),
        &framework_path,
        "rescue",
        Some(script),
    )
    .unwrap();

    temp_script_path.path().to_owned()
}

pub fn deadline_secs(secs: u64) -> Instant {
    Instant::now()
        .checked_add(Duration::from_secs(secs))
        .expect("no deadline")
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
        .unwrap_or_else(|err| {
            let lsof_output = Command::new("lsof").arg("-i").output().unwrap();
            panic!(
                "wait_until_healthy failed. lsof -i: {:?}: {}",
                lsof_output, err
            );
        });
    info!("Validator restart health check passed");

    let connectivity_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_CONNECTIVITY_WAIT_SECS))
        .context("can't get new deadline")?;
    validator
        .wait_for_connectivity(expected_to_connect, connectivity_deadline)
        .await?;
    info!("Validator restart connectivity check passed");
    Ok(())
}

/// If there are multiple modules being upgraded only one of the modules (the
/// first) needs to be included in the proposal.
/// The transaction script which upgrades the first module, also sets the
/// transaction hash for the subsequent module needed to be upgraded.
/// these hashes are produced offline during the framework upgrade builder
/// workflow.
pub async fn upgrade_multiple_impl(
    dir_path: &str,
    modules: Vec<&str>,
    smoke: &mut LibraSmoke,
) -> anyhow::Result<()> {
    upgrade_fixtures::testsuite_maybe_warmup_fixtures();

    let d = diem_temppath::TempPath::new();

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut smoke.swarm, 0, Some(d.path().to_owned()))
            .context("could not init validator config")?;

    // This step should fail. The view function does not yet exist in the system address.
    // we will upgrade a new binary which will include this function.
    let query_res = query_view::get_view(
        &smoke.client(),
        "0x1::all_your_base::are_belong_to",
        None,
        None,
    )
    .await;
    assert!(query_res.is_err(), "expected all_your_base to fail");

    ///// NOTE THERE ARE MULTIPLE STEPS, we are getting the artifacts for the
    // first step. This is what sets the governance in motion
    // we do not need to submit proposals for each subsequent step.
    // that's because the resolution of the the first step, already
    // includes the hash of the second step, which gets stored in
    // advance of the user resolving the step 2 with its transaction.

    //////////// PROPOSAL ////////////
    // Set up governance proposal, just with first module
    let script_dir = upgrade_fixtures::fixtures_path()
        .join(dir_path)
        .join(modules[0]); // take first module usually "1-move-stdlib"
    assert!(script_dir.exists(), "can't find upgrade fixtures");

    let mut cli = TxsCli {
        subcommand: Some(Governance(Propose {
            proposal_script_dir: script_dir.clone(),
            metadata_url: "http://allyourbase.com".to_string(),
        })),
        mnemonic: None,
        test_private_key: Some(smoke.encoded_pri_key.clone()),
        chain_name: Some(NamedChain::TESTING),
        config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(smoke.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::prod_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli.run()
        .await
        .context("cli could not send upgrade proposal")?;

    //////////// VOTING ////////////
    println!(">>> Vals voting on proposal");
    for val_key in smoke.validator_private_keys.iter() {
        cli.subcommand = Some(Governance(Vote {
            proposal_id: 4,
            should_fail: false,
        }));
        cli.test_private_key = Some(val_key.clone());
        cli.run().await.context("cli could not send vote")?;
    }

    // ensure the proposal is passing
    let query_res = query_view::get_view(
        &smoke.client(),
        "0x1::diem_governance::get_proposal_state",
        None,
        Some("4".to_string()),
    )
    .await?;

    assert!(
        query_res[0].as_str().unwrap() == "1",
        "proposal state should be 1, passing"
    );

    let query_res = query_view::get_view(
        &smoke.client(),
        "0x1::voting::is_voting_closed",
        Some("0x1::governance_proposal::GovernanceProposal".to_string()),
        Some("0x1, 4".to_string()),
    )
    .await?;

    assert!(query_res[0].as_bool().unwrap(), "expected to be closed");

    // Note, if there isn't a pause here, the next request might happen on the same on-chain clock seconds as the previous.
    // this will intentionally cause a failure since it's designed to prevent "atomic" transactions which can manipulate governance (flash loans)
    std::thread::sleep(std::time::Duration::from_secs(3));

    let query_res = query_view::get_view(
        &smoke.client(),
        "0x1::diem_governance::get_can_resolve",
        None,
        Some("4".to_string()),
    )
    .await?;
    assert!(
        query_res[0].as_bool().unwrap(),
        "expected to be able to resolve"
    );

    let query_res = query_view::get_view(
        &smoke.client(),
        "0x1::diem_governance::get_approved_hash",
        None,
        Some("4".to_string()),
    )
    .await?;

    let expected_hash = std::fs::read_to_string(script_dir.join("script_sha3")).unwrap();
    assert!(
        query_res[0].as_str().unwrap().contains(&expected_hash),
        "expected this script hash, did you change the fixtures?"
    );

    //////////// RESOLVE ////////////
    cli.test_private_key = Some(smoke.encoded_pri_key.clone());
    for name in modules {
        ///////// SHOW TIME, RESOLVE EACH STEP ////////
        let script_dir = upgrade_fixtures::fixtures_path().join(dir_path).join(name);
        cli.subcommand = Some(Governance(Resolve {
            proposal_id: 4,
            proposal_script_dir: script_dir,
        }));
        cli.run()
            .await
            .map_err(|e| e.context(format!("cannot resolve proposal at step {name}")))?;
    }

    Ok(())
}
