use diem_smoke_test::smoke_test_environment::new_local_swarm_with_diem;
use diem_forge::Swarm;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn _meta_can_start_swarm() {
    let mut swarm = new_local_swarm_with_diem(4).await;
    let _info = swarm.diem_public_info();
}
