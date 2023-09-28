use crate::libra_smoke::LibraSmoke;

use diem_forge::Swarm;
use libra_framework::release::ReleaseTarget;
use smoke_test::smoke_test_environment::new_local_swarm_with_release;


/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_can_start_swarm() {
    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let mut info = swarm.diem_public_info();

    let a = info.random_account();
    info.create_user_account(a.public_key()).await.expect("cannot create account");

}

/// testing the LibraSmoke abstraction can load
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_create_libra_smoke() {
    let _s = LibraSmoke::new(Some(3))
        .await
        .expect("cannot start libra swarm");
}
