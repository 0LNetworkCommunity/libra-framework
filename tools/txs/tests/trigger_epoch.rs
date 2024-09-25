use diem_forge::Swarm;
use libra_query::query_view;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::{submit_transaction::Sender, txs_cli_governance::GovernanceTxs};
use libra_cached_packages::libra_stdlib;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// Test triggering a new epoch
// Scenario: We want to trigger a new epoch using the TriggerEpoch command
// We will assume that triggering an epoch is an operation that we can test in a single node testnet
async fn trigger_epoch() -> anyhow::Result<()> {
    // create libra swarm and get app config for the validator
    let mut ls = LibraSmoke::new(Some(1), None)
        .await
        .expect("could not start libra smoke");
    let val_app_cfg = ls.first_account_app_cfg()?;

    //////// FLIP BIT ////////

    // testnet root can set the epoch boundary bit
    let mut public_info = ls.swarm.diem_public_info();

    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::epoch_boundary_smoke_enable_trigger());

    let demo_txn = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info
        .client()
        .submit_and_wait(&demo_txn)
        .await
        .expect("could not send demo tx");

    // create a Sender using the validator's app config
    let mut validator_sender = Sender::from_app_cfg(&val_app_cfg, None).await?;

    let before_trigger_epoch_query_res = query_view::get_view(
        &ls.client(),
        "0x1::reconfiguration::get_current_epoch",
        None,
        None,
    )
    .await
    .expect("Query failed: get epoch failed");

    assert_eq!(
        &before_trigger_epoch_query_res.as_array().unwrap()[0],
        "2",
        "Epoch is not 2"
    );

    //////// TRIGGER THE EPOCH ////////
    // The TriggerEpoch command does not require arguments,
    // so we create it directly and attempt to run it.
    let trigger_epoch_cmd = GovernanceTxs::EpochBoundary;

    // run the command and assert it succeeds
    let res = trigger_epoch_cmd.run(&mut validator_sender).await;
    assert!(res.is_ok(), "Failed to trigger new epoch: {:?}", res);

    let before_trigger_epoch_query_res = query_view::get_view(
        &ls.client(),
        "0x1::reconfiguration::get_current_epoch",
        None,
        None,
    )
    .await
    .expect("Query failed: get epoch failed");

    assert!(&before_trigger_epoch_query_res.as_array().unwrap()[0] == "3", "Epoch is not 3");

    Ok(())
}
