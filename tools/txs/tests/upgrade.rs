use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Upgrade},
    txs_cli_upgrade::UpgradeTxs,
};

/// Testing that we can upgrade the chain framework using txs tools.
/// Note: We have another upgrade meta test in ./smoke-tests
/// We assume a built transaction script for upgrade in tests/fixtures/test_upgrade.
/// 1. a validator can submit a proposal with txs
/// 2. the validator can vote for the proposal
/// 3. check that the proposal is resolvable
/// 4. resolve a propsosal by sending the upgrade payload.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade() {
    let mut s = LibraSmoke::new(None).await.expect("can't start swarm");
    let this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?.unwrap();
    let script_dir = this_path.join("tests/fixtures/test_upgrade");
    assert!(script_dir.exists(), "can't find upgrade fixtures");

    let mut cli = TxsCli {
        subcommand: Some(Upgrade(
            UpgradeTxs,
            Propose {
                proposal_script_dir: script_dir,
                metadata_url: "http://allyourbase.com",
            },
        )),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: None,
        url: Some(s.api_endpoint.clone()),
        gas_max: None,
        gas_unit_price: None,
    };

    cli.run().await.expect("cli could not send upgrade proposal");

}
