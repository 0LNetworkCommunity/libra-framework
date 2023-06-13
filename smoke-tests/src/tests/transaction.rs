use aptos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use aptos_forge::Swarm;
use libra_txs::txs_cli::TxsCli;
/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn can_send_txs_cli() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let _info = swarm.aptos_public_info();

    TxsCli {

    }
}
