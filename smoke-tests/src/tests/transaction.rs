use aptos_forge::Node;
use aptos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use libra_txs::txs_cli::TxsCli;
use libra_txs::txs_cli::Subcommand::Transfer;
use zapatos_crypto::traits::ValidCryptoMaterialStringExt;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn can_send_txs_cli() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let swarm = new_local_swarm_with_release(2, release).await;
    // swarm has started

    // use the 0th node to send a tx
    let node = swarm.validators().next().unwrap();
    let pri_key = node.account_private_key().as_ref().unwrap().private_key();

    let t = TxsCli {
      subcommand: Some(Transfer {
        to_account: swarm.validators().nth(1).unwrap().peer_id(), // sending to sedond node.
        amount: 1,
      }),
      mnemonic: None,
      private_key: Some(pri_key.to_encoded_string().expect("cannot decode pri key")),
      url: Some(node.rest_api_endpoint()),
      max_gas: None,
      gas_unit_price: None,
    };

    t.run().await.unwrap();
}
