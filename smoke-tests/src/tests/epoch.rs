use diem_forge::Swarm;

use crate::libra_smoke::LibraSmoke;

/// testing epoch changes after 2 mins
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn meta_create_libra_smoke() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new(None)
        .await
        .expect("cannot start libra swarm");
    let mut p = s.swarm.diem_public_info();
    let prev_epoch = {
        let client = p.client();
        let li = client.get_ledger_information().await?;
        li.inner().epoch
    };

    let state = p.reconfig().await;

    assert!(prev_epoch < state.epoch, "epoch did not advance on reconfig()");

    Ok(())
}
