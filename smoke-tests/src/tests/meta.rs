use libra_framework::release::ReleaseTarget;
use diem_forge::Swarm;
use diem_smoke_test::smoke_test_environment::new_local_swarm_with_release;

use crate::libra_smoke::LibraSmoke;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_can_start_swarm() {
    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let _info = swarm.diem_public_info();
}

/// testing the LibraSmoke abstraction can load
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_create_libra_smoke() {
    // let release = ReleaseTarget::Head.load_bundle().unwrap();
    // let mut swarm = new_local_swarm_with_release(4, release).await;
    // let _info = swarm.diem_public_info();
    let _s = LibraSmoke::new(None)
        .await
        .expect("cannot start libra swarm");
}
