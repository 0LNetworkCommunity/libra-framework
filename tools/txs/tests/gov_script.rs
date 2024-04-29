use std::path::PathBuf;
use std::str::FromStr;

use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Governance},
    txs_cli_governance::GovernanceTxs::{Propose, Resolve, Vote},
};
use libra_types::legacy_types::app_cfg::TxCost;

/// Testing that we can upgrade the chain framework using txs tools.
/// Note: We have another upgrade meta test in ./smoke-tests
/// We assume a built transaction script for upgrade in tests/fixtures/test_upgrade.
/// 1. a validator can submit a proposal with txs
/// 2. the validator can vote for the proposal
/// 3. check that the proposal is resolvable
/// 4. resolve a propsosal by sending the upgrade payload.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_gov_script() {
    let d = diem_temppath::TempPath::new();

    let mut s = LibraSmoke::new(Some(2))
        .await
        .expect("could not start libra smoke");

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    let this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let script_dir = this_path.join("tests/fixtures/governance_script_template");
    assert!(script_dir.exists(), "can't find upgrade fixtures");

    let mut cli = TxsCli {
        subcommand: Some(Governance(Propose {
            proposal_script_dir: script_dir.clone(),
            metadata_url: "http://allyourbase.com".to_string(),
        })),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_baseline_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli.run()
        .await
        .expect("cli could not send upgrade proposal");

    // ALICE VOTES
    cli.subcommand = Some(Governance(Vote {
        proposal_id: 0,
        should_fail: false,
    }));
    cli.run().await.unwrap();

    let _query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_proposal_state",
        None,
        Some("0".to_string()),
    )
    .await
    .unwrap();

    let _query_res = query_view::get_view(
        &s.client(),
        "0x1::voting::is_voting_closed",
        Some("0x1::governance_proposal::GovernanceProposal".to_string()),
        Some("0x1, 0".to_string()),
    )
    .await
    .unwrap();

    let _query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_can_resolve",
        None,
        Some("0".to_string()), //Some(format!("{}u64", id)),
    )
    .await
    .unwrap();

    let _query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_approved_hash",
        None,
        Some("0".to_string()),
    )
    .await
    .unwrap();

    // Now try to resolve upgrade
    cli.subcommand = Some(Governance(Resolve {
        proposal_id: 0,
        proposal_script_dir: script_dir,
    }));
    cli.run().await.unwrap();
}
