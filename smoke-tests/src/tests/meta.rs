use crate::libra_smoke::LibraSmoke;

use diem_forge::Swarm;
use libra_cached_packages::libra_stdlib;
use libra_framework::release::ReleaseTarget;
use smoke_test::smoke_test_environment::new_local_swarm_with_release;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_can_start_swarm() {
    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let mut public_info = swarm.diem_public_info();

    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::demo_print_this());

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
async fn meta_create_libra_smoke() {
    let _s = LibraSmoke::new(Some(3))
        .await
        .expect("cannot start libra swarm");
}
