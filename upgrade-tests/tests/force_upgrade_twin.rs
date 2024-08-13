mod support;

use anyhow::Context;
use libra_framework::release::ReleaseTarget;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_twin_tests::runner::Twin;
// Here we are testing how a Twin modified swarm responds to an upgrade
// upgrades are being applied against Mainnet data which is recovered by a snapshot.

/// Meta test: Upgrade all modules on a no-op dummy database (same as swarm)
/// should have same behavior as a normal swarm.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn twin_test_all_upgrades_dummy() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;

    // Is not trying to restore from an actual Twin, hence None
    // just a meta integration test
    Twin::make_twin_swarm(&mut s, None, false).await?;

    support::upgrade_multiple_impl(
        &mut s,
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
    )
    .await?;
    Ok(())
}
