use std::str::FromStr;
use std::{fs, path::PathBuf};

use diem_types::chain_id::NamedChain;
use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Upgrade},
    txs_cli_upgrade::UpgradeTxs::{Propose, Resolve, Vote},
};
use libra_types::legacy_types::app_cfg::TxCost;

/// Testing that we can upgrade the chain framework using txs tools.
/// Note: We have another upgrade meta test in ./smoke-tests
/// We assume a built transaction script for upgrade in tests/fixtures/test_upgrade.
/// 1. a validator can submit a proposal with txs
/// 2. the validator can vote for the proposal
/// 3. check that the proposal is resolvable
/// 4. resolve a propsosal by sending the upgrade payload.
/// 5. Check that the new function all_your_base can be called
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade_single_step() {
    let d = diem_temppath::TempPath::new();

    let mut s = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");
    // let this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let script_dir = get_package_path();
    assert!(script_dir.exists(), "can't find upgrade fixtures");

    // This step should fail. The view function does not yet exist in the system address.
    // we will upgrade a new binary which will include this function.
    let query_res =
        query_view::get_view(&s.client(), "0x1::all_your_base::are_belong_to", None, None).await;
    assert!(query_res.is_err(), "expected all_your_base to fail");

    let mut cli = TxsCli {
        subcommand: Some(Upgrade(Propose {
            proposal_script_dir: script_dir.clone(),
            metadata_url: "http://allyourbase.com".to_string(),
        })),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: Some(NamedChain::TESTING),
        config_path: Some(d.path().to_owned().join("libra.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_critical_txs_cost()),
        estimate_only: false,
    };

    cli.run()
        .await
        .expect("cli could not send upgrade proposal");

    // ALICE VOTES
    cli.subcommand = Some(Upgrade(Vote {
        proposal_id: 0,
        should_fail: false,
    }));
    cli.run().await.expect("alice votes on prop 0");

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_proposal_state",
        None,
        Some("0".to_string()),
    )
    .await
    .unwrap();

    assert!(
        query_res[0].as_str().unwrap() == "1",
        "proposal state should be 1, passing"
    );

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::voting::is_voting_closed",
        Some("0x1::governance_proposal::GovernanceProposal".to_string()),
        Some("0x1, 0".to_string()),
    )
    .await
    .unwrap();

    assert!(query_res[0].as_bool().unwrap(), "expected to be closed");

    // Note, if there isn't a pause here, the next request might happen on the same on-chain clock seconds as the previous.
    // this will intentionally cause a failure since it's designed to prevent "atomic" transactions which can manipulate governance (flash loans)
    std::thread::sleep(std::time::Duration::from_secs(4));

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_can_resolve",
        None,
        Some("0".to_string()),
    )
    .await
    .unwrap();
    assert!(
        query_res[0].as_bool().unwrap(),
        "expected to be able to resolve"
    );

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_approved_hash",
        None,
        Some("0".to_string()),
    )
    .await
    .unwrap();

    let expected_hash = fs::read_to_string(script_dir.join("script_sha3")).unwrap();
    assert!(
        query_res[0].as_str().unwrap().contains(&expected_hash),
        "expected this script hash, did you change the fixtures?"
    );

    ///////// SHOWTIME ////////
    // Now try to resolve upgrade
    cli.subcommand = Some(Upgrade(Resolve {
        proposal_id: 0,
        proposal_script_dir: script_dir,
    }));
    cli.run().await.expect("alice cannot resolve proposal");
    //////////////////////////////

    let query_res =
        query_view::get_view(&s.client(), "0x1::all_your_base::are_belong_to", None, None)
            .await
            .unwrap();
    assert!(&query_res.as_array().unwrap()[0]
        .as_str()
        .unwrap()
        .contains("7573"));
}

fn get_package_path() -> PathBuf {
    let this_crate = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_crate
    .join("tests")
    .join("fixtures")
    .join("upgrade-single-lib")
    .join("1-move-stdlib")
}

