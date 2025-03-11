//! test trigger epoch

use std::sync::{Arc, Mutex};
use std::time::Duration;

use diem_forge::Swarm;
use libra_cached_packages::libra_stdlib;
use libra_query::query_view;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::{
    submit_transaction::Sender, txs_cli_governance::GovernanceTxs, txs_cli_stream::StreamTxs,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// Test triggering a new epoch
// Scenario: We want to trigger a new epoch using the TriggerEpoch command
// We will assume that triggering an epoch is an operation that we can test in a single node testnet
async fn sync_trigger_epoch() -> anyhow::Result<()> {
    // create libra swarm and get app config for the validator
    let mut ls = LibraSmoke::new(Some(1), None)
        .await
        .expect("could not start libra smoke");
    let val_app_cfg = ls.first_account_app_cfg()?;

    let before_trigger_epoch_query_res = query_view::get_view(
        &ls.client(),
        "0x1::epoch_helper::get_current_epoch",
        None,
        None,
    )
    .await
    .expect("query failed: get epoch failed");

    // TODO: why is it that smoke tests start on epoch 2?
    assert_eq!(
        &before_trigger_epoch_query_res.as_array().unwrap()[0],
        "2",
        "epoch should be 2"
    );

    //////// FLIP BIT ////////

    helper_set_enable_trigger(&mut ls).await;

    //////// TRIGGER THE EPOCH ////////
    // The TriggerEpoch command does not require arguments,
    // so we create it directly and attempt to run it.
    let trigger_epoch_cmd = GovernanceTxs::EpochBoundary;
    // create a Sender using the validator's app config
    let mut validator_sender = Sender::from_app_cfg(&val_app_cfg, None).await?;

    // run the command and assert it succeeds
    let res = trigger_epoch_cmd.run(&mut validator_sender).await;
    assert!(res.is_ok(), "Failed to trigger new epoch: {:?}", res);

    let after_trigger_epoch_query_res = query_view::get_view(
        &ls.client(),
        "0x1::epoch_helper::get_current_epoch",
        None,
        None,
    )
    .await
    .expect("Query failed: get epoch failed");

    assert!(
        &after_trigger_epoch_query_res.as_array().unwrap()[0] == "3",
        "epoch should be 3"
    );

    Ok(())
}

#[tokio::test (flavor = "multi_thread", worker_threads = 2)]
/// Test triggering a new epoch
// Scenario: We want to trigger a new epoch using the TriggerEpoch command
// We will assume that triggering an epoch is an operation that we can test in a single node testnet
async fn background_trigger_epoch() -> anyhow::Result<()> {
    // create libra swarm and get app config for the validator
    let mut ls = LibraSmoke::new(Some(1), None)
        .await
        .expect("could not start libra smoke");

    let before_trigger_epoch_query_res = query_view::get_view(
        &ls.client(),
        "0x1::epoch_helper::get_current_epoch",
        None,
        None,
    )
    .await
    .expect("query failed: get epoch failed");

    // TODO: why is it that smoke tests start on epoch 2?
    assert_eq!(
        &before_trigger_epoch_query_res.as_array().unwrap()[0],
        "2",
        "epoch should be 2"
    );

    //////// TRIGGER THE EPOCH ////////
    // The TriggerEpoch command does not require arguments,
    // so we create it directly and attempt to run it.
    let trigger_epoch_cmd = StreamTxs::EpochTickle { delay: Some(5) };
    // create a Sender using the validator's app config
    let val_app_cfg = ls.first_account_app_cfg()?;
    let validator_sender = Sender::from_app_cfg(&val_app_cfg, None).await?;
    let wrapped_sender = Arc::new(Mutex::new(validator_sender));

    // run the txs tool in background in stream

    tokio::spawn(async move {
        trigger_epoch_cmd.start(wrapped_sender).await;
    });


    //////// FLIP BIT ////////
    std::thread::sleep(Duration::from_secs(10));

    helper_set_enable_trigger(&mut ls).await;

    std::thread::sleep(Duration::from_secs(20));

    // now the background service should succeed in triggering epoch.

    let after_trigger_epoch_query_res = query_view::get_view(
        &ls.client(),
        "0x1::epoch_helper::get_current_epoch",
        None,
        None,
    )
    .await
    .expect("Query failed: get epoch failed");

    assert!(
        &after_trigger_epoch_query_res.as_array().unwrap()[0] == "3",
        "epoch should be 3"
    );

    Ok(())
}

// helper for the testnet root to enable epoch boundary trigger
async fn helper_set_enable_trigger(ls: &mut LibraSmoke) {
    println!("enable trigger");

    let mut public_info = ls.swarm.diem_public_info();

    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::epoch_boundary_smoke_enable_trigger());

    let enable_trigger_tx = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info
        .client()
        .submit_and_wait(&enable_trigger_tx)
        .await
        .expect("could not send demo tx");
    println!("testnet root account sets epoch boundary trigger");
}
