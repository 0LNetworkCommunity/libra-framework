use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use libra_cached_packages::aptos_stdlib::EntryFunctionCall::DemoPrintThis;
use zapatos_forge::Swarm;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn batch_send() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let mut info = swarm.aptos_public_info();
    
    let mut account1 = info.create_and_fund_user_account(10000).await.expect("create_and_fund_user_account failed");


     let txn = account1.sign_with_transaction_builder(
        info.transaction_factory()
            .payload(DemoPrintThis{}.encode()),
    );

    let client = info.client();
    let res = client.simulate(&txn).await.expect("simulate failed");
    dbg!(&res);

}
