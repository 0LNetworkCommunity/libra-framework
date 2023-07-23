
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::txs_cli::{
  TxsCli,
  TxsSub::Transfer,
};

/// Testing that we can send the minimal transaction: a transfer from one existing validator to another.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn can_send_txs_cli() {
    let s = LibraSmoke::new(Some(2)).await.expect("can't start swarm");

    let t = TxsCli {
      subcommand: Some(Transfer {
        to_account: s.swarm.validators().nth(1).unwrap().peer_id(), // sending to second node.
        amount: 1,
      }),
      mnemonic: None,
      test_private_key: Some(s.encoded_pri_key),
      chain_id: None,
      config_path: None,
      url: Some(s.api_endpoint),
      gas_max: None,
      gas_unit_price: None,
    };

    t.run().await.unwrap();
}
