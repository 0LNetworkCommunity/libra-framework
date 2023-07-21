use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use zapatos_forge::Swarm;
use libra_query::query_type::QueryType;
use libra_types::gas_coin::SlowWalletBalance;

/// Testing the query library
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn libra_query_test() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(1, release).await;
    let val_acct = swarm.validators().next().unwrap().peer_id();

    let info = swarm.aptos_public_info();
    let c = info.client();

    let q = QueryType::Balance { account: val_acct };
    match q.query_to_json(Some(c.to_owned())).await {
      Ok(v) => {
        println!("v: {:?}", v);
        let b: SlowWalletBalance = serde_json::from_value(v).unwrap();
        assert!(b.unlocked == 10000000000);
        assert!(b.total == 10000000000);
      },
      Err(e) => {
        println!("e: {:?}", e);
        assert!(false);
      }
    }
}
