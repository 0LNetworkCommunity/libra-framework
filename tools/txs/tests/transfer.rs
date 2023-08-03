use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::txs_cli::{TxsCli, TxsSub::Transfer};

/// Testing that we can send the minimal transaction: a transfer from one existing validator to another.
/// Case 1: send to an existing account: another genesis validator
/// Case 2: send to an account which does not yet exist, and the account gets created on chain.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_transfer() {
    let mut s = LibraSmoke::new(Some(2)).await.expect("can't start swarm");

    // 1. simple case: account already exitsts
    let recipient = s.swarm.validators().nth(1).unwrap().peer_id(); // sending to second node.
    let mut cli = TxsCli {
        subcommand: Some(Transfer {
            to_account: recipient,
            amount: 1,
        }),
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
        .expect("cli could not send to existing account");

    //2. Account does not yet exist.
    cli.subcommand = Some(Transfer {
        to_account: s.marlon_rando().address().clone(),
        amount: 1,
    });

    cli.run().await.expect("cli could not create new account");

    //TODO: check the balance
}
