use std::path::PathBuf;
use std::str::FromStr;

use libra_types::exports::ValidCryptoMaterialStringExt;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_query::query_view;
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Upgrade},
    txs_cli_upgrade::UpgradeTxs::{Propose, Resolve, Vote},
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
    let mut s = LibraSmoke::new(Some(3)).await.expect("can't start swarm");
    let this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let script_dir = this_path.join("tests/fixtures/test_upgrade");
    assert!(script_dir.exists(), "can't find upgrade fixtures");

    let mut cli = TxsCli {
        subcommand: Some(Upgrade(Propose {
            proposal_script_dir: script_dir.clone(),
            metadata_url: "http://allyourbase.com".to_string(),
        })),
        mnemonic: None,
        test_private_key: Some(s.encoded_pri_key.clone()),
        chain_id: None,
        config_path: None,
        url: Some(s.api_endpoint.clone()),
        gas_max: None,
        gas_unit_price: None,
    };

    cli.run()
        .await
        .expect("cli could not send upgrade proposal");

    // // VOTE on an ID that does not exist should fail
    // cli.subcommand = Some(Upgrade(Vote {
    //     proposal_id: 1,
    //     should_fail: false,
    // }));
    // assert!(
    //     cli.run().await.is_err(),
    //     "proposal is not expected to exist"
    // );

    cli.subcommand = Some(Upgrade(Vote {
        proposal_id: 0,
        should_fail: false,
    }));

    // TODO: do this in a sane way.

    let vals_keys = s.swarm.validators()
    .map(|node| {
      node.account_private_key()
      .as_ref()
      .unwrap()
      .private_key()
      .to_encoded_string()
      .expect("cannot decode pri key")
    }).collect::<Vec<String>>();
    //ALICE

    let mut key_iter = vals_keys.iter();



    cli.test_private_key = Some(key_iter.next().unwrap().to_owned());
    cli.run().await.expect("could not send vote");


    // BOB

    let bob = s.swarm.validators().nth(1).unwrap().peer_id();
    s.mint(bob, 5_000_000_000).await.unwrap();
    cli.test_private_key = Some(key_iter.next().unwrap().to_owned());
    cli.run().await.expect("could not send vote");

    // CAROL
    let carol = s.swarm.validators().nth(2).unwrap().peer_id();
    s.mint(carol, 5_000_000_000).await.unwrap();
    cli.test_private_key = Some(key_iter.next().unwrap().to_owned());
    cli.run().await.expect("could not send vote");
    // let node = vals_iter.next().unwrap();

    // let pri_key = node.account_private_key()
    //   .as_ref()
    //   .unwrap()
    //   .private_key()
    //   .to_encoded_string()
    //   .expect("cannot decode pri key");

    // cli.test_private_key = Some(pri_key);
    // cli.run().await.expect("could not send vote");

    // // This proposal exists
    // cli.subcommand = Some(Upgrade(Resolve {
    //     proposal_id: 0,
    //     proposal_script_dir: script_dir,
    // }));
    // cli.run().await.expect("can't resolve yet");

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::aptos_governance::get_proposal_state",
        None,
        Some("0".to_string()), //Some(format!("{}u64", id)),
    )
    .await.unwrap();
    dbg!(&query_res);

    let query_res = query_view::get_view(
        &s.client(),
        "0x1::voting::is_voting_closed",
        Some("0x1::governance_proposal::GovernanceProposal".to_string()),
        Some("0x1, 0".to_string()), //Some(format!("{}u64", id)),
    )
    .await.unwrap();
    dbg!(&query_res);
}
