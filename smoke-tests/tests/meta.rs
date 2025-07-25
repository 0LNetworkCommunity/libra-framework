use libra_smoke_tests::libra_smoke::LibraSmoke;

use diem_forge::Swarm;
use libra_cached_packages::libra_stdlib;
use libra_framework::release::ReleaseTarget;
use smoke_test::smoke_test_environment::new_local_swarm_with_release;

/// testing that we can get a swarm up of 1 node with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_can_start_swarm() {
    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(1, release, None).await;
    let mut public_info = swarm.diem_public_info();

    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::diem_governance_smoke_trigger_epoch());

    let demo_txn = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info
        .client()
        .submit_and_wait(&demo_txn)
        .await
        .expect("could not send demo tx");
}

/// testing the LibraSmoke abstraction can load
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_create_libra_smoke_single() {
    let _s = LibraSmoke::new(Some(1), None)
        .await
        .expect("cannot start libra swarm");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn meta_create_libra_smoke_multi() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new(Some(5), None)
        .await
        .expect("cannot start libra swarm");

    let c = s.client();
    let res = c
        .get_account_resource(
            "0x1".parse().unwrap(),
            "0x1::epoch_boundary::BoundaryStatus",
        )
        .await?;

    // check that the genesis epochs 0, 1, transitioned safely, and epoch 2 has
    // all the validators
    let t = res.inner().as_ref().unwrap();
    let o = t.data.as_object().expect("no epoch_boundary response");
    let num = o
        .get("incoming_filled_seats")
        .expect("no incoming filled seats value")
        .as_str()
        .unwrap()
        .parse::<u64>()?;
    assert_eq!(num, 5);

    Ok(())
}
