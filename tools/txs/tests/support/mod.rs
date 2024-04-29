use anyhow::Context;
use diem_types::chain_id::NamedChain;
use libra_framework::{release::ReleaseTarget, upgrade_fixtures};
use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Governance},
    txs_cli_governance::GovernanceTxs::{Propose, Resolve, Vote},
};
use libra_types::legacy_types::app_cfg::TxCost;

/// If there are multiple modules being upgraded only one of the modules (the
/// first) needs to be included in the proposal.
/// The transaction script which upgrades the first module, also sets the
/// transaction hash for the subsequent module needed to be upgraded.
/// these hashes are produced offline during the framework upgrade builder
/// workflow.
pub async fn upgrade_multiple_impl(
    dir_path: &str,
    modules: Vec<&str>,
    prior_release: ReleaseTarget,
) -> anyhow::Result<()> {
    upgrade_fixtures::testsuite_maybe_warmup_fixtures();

    let d = diem_temppath::TempPath::new();

    let mut s = LibraSmoke::new_with_target(Some(1), prior_release)
        .await
        .context("could not start libra smoke")?;

    let (_, _app_cfg) =
        configure_validator::init_val_config_files(&mut s.swarm, 0, d.path().to_owned())
            .await
            .context("could not init validator config")?;

    // This step should fail. The view function does not yet exist in the system address.
    // we will upgrade a new binary which will include this function.
    let query_res =
        query_view::get_view(&s.client(), "0x1::all_your_base::are_belong_to", None, None).await;
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
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: Some(NamedChain::TESTING),
        config_path: Some(d.path().to_owned().join("libra-cli-config.yaml")),
        url: Some(s.api_endpoint.clone()),
        tx_profile: None,
        tx_cost: Some(TxCost::default_critical_txs_cost()),
        estimate_only: false,
        legacy_address: false,
    };

    cli.run()
        .await
        .context("cli could not send upgrade proposal")?;

    //////////// VOTING ////////////

    // ALICE VOTES
    cli.subcommand = Some(Governance(Vote {
        proposal_id: 0,
        should_fail: false,
    }));
    cli.run().await.context("alice votes on prop 0")?;

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_proposal_state",
        None,
        Some("0".to_string()),
    )
    .await?;

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
    .await?;

    assert!(query_res[0].as_bool().unwrap(), "expected to be closed");

    // Note, if there isn't a pause here, the next request might happen on the same on-chain clock seconds as the previous.
    // this will intentionally cause a failure since it's designed to prevent "atomic" transactions which can manipulate governance (flash loans)
    std::thread::sleep(std::time::Duration::from_secs(3));

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::diem_governance::get_can_resolve",
        None,
        Some("0".to_string()),
    )
    .await?;
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
    .await?;

    let expected_hash = std::fs::read_to_string(script_dir.join("script_sha3")).unwrap();
    assert!(
        query_res[0].as_str().unwrap().contains(&expected_hash),
        "expected this script hash, did you change the fixtures?"
    );

    //////////// RESOLVE ////////////

    for name in modules {
        ///////// SHOW TIME, RESOLVE EACH STEP ////////

        let script_dir = upgrade_fixtures::fixtures_path().join(dir_path).join(name);

        cli.subcommand = Some(Governance(Resolve {
            proposal_id: 0,
            proposal_script_dir: script_dir,
        }));
        cli.run()
            .await
            .map_err(|e| e.context("cannot resolve proposal at step {name}"))?;
    }

    //////////// VERIFY SUCCESS ////////////
    let query_res =
        query_view::get_view(&s.client(), "0x1::all_your_base::are_belong_to", None, None)
            .await
            .context("no all_your_base module found")?;
    assert!(&query_res.as_array().unwrap()[0]
        .as_str()
        .unwrap()
        .contains("7573")); // bytes for "us"
    Ok(())
}
