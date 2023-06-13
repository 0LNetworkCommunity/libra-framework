use crate::helpers::{mint_libra, get_libra_balance} ;
use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use zapatos_forge::Swarm;
use zapatos_sdk::types::LocalAccount;

#[tokio::test]
// let's check that this test environment produces same coins as expected in unit tests, and we have the tools to mint and test balances
async fn sanity_balances() {
    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let v = swarm.validators_mut().next().unwrap();
    let pri_key = v.account_private_key().as_ref().unwrap();
    let address = v.peer_id().to_owned();
    let _account = LocalAccount::new(v.peer_id(), pri_key.private_key(), 0);
    let mut public_info: zapatos_forge::AptosPublicInfo = swarm.aptos_public_info();

    let balance = public_info.client()
        .get_account_balance(address)
        .await
        .unwrap()
        .into_inner();

    // dbg!(&balance.coin.value.0);
    assert!(1 == balance.coin.value.0);

    // the `core address` sudo account for tests can mint vendor coin
    public_info.mint(address, 10_000_000).await.unwrap();

    let balance = public_info.client()
        .get_account_balance(address)
        .await
        .unwrap()
        .into_inner();

    // dbg!(&balance.coin.value.0);
    assert!(10000001 == balance.coin.value.0);

    let gas_balance = get_libra_balance(&public_info.client(), address)
        .await
        .unwrap()
        .into_inner();
    // dbg!(&gas_balance);
    // dbg!(&gas_balance.coin.value.0);
    assert!(1 == gas_balance.coin.value.0);


    mint_libra(&mut public_info, address, 10_000_000).await.unwrap();

    let gas_balance = get_libra_balance(&public_info.client(), address)
      .await
      .unwrap()
      .into_inner();
    // dbg!(&gas_balance);
    // dbg!(&gas_balance.coin.value.0);

    assert!(10000001 == gas_balance.coin.value.0);

}
