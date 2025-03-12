//! test commit reveal
use std::time::Duration;
use libra_query::query_view;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::stream::bid_commit_reveal::PofBidArgs;
use libra_txs::{submit_transaction::Sender, txs_cli_stream::StreamTxs};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
/// Test triggering a new epoch
// Scenario: We want to trigger a new epoch using the TriggerEpoch command
// We will assume that triggering an epoch is an operation that we can test in a single node testnet
async fn background_commit_reveal() -> anyhow::Result<()> {
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

    let test_bid_amount = 1010;

    //////// TRIGGER THE EPOCH ////////
    // The TriggerEpoch command does not require arguments,
    // so we create it directly and attempt to run it.
    let commit_reveal_cmd = StreamTxs::ValBid(PofBidArgs {
        net_reward: test_bid_amount,
        delay: Some(5),
    });
    // create a Sender using the validator's app config
    let val_app_cfg = ls.first_account_app_cfg()?;
    let mut validator_sender = Sender::from_app_cfg(&val_app_cfg, None).await?;
    let client = validator_sender.client().clone();
    let validator_addr = validator_sender.local_account.address();

    // run the txs tool in background in stream mode

    let h = tokio::spawn(async move {
        commit_reveal_cmd.start(&mut validator_sender).await.unwrap();
    });

    // WAIT FOR END OF EPOCH chain_id==4 is 30 secs
    std::thread::sleep(Duration::from_secs(30));
    h.abort();

    match libra_query::chain_queries::validator_committed_bid(&client, validator_addr).await {
        Ok(onchain_bid) => {
            assert!(test_bid_amount == onchain_bid, "incorrect bid found");
            println!("Validator committed bid successfully.");
        }
        Err(e) => {
            assert!(false, "Failed to commit bid: {:?}", e);
        }
    }

    Ok(())
}
