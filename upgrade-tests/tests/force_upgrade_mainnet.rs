mod support;

use anyhow::Context;
use libra_framework::release::ReleaseTarget;
use libra_smoke_tests::libra_smoke::LibraSmoke;

/////// TEST ARBITRARY UPGRADES ///////
// do the same as above, but use the "arbitrary" upgrade policy to force an
// upgrade.
//
/// Force upgrade Libra
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn force_upgrade_mainnet_libra() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;
    support::upgrade_multiple_impl(
        &mut s,
        "upgrade-single-lib-force",
        vec!["1-libra-framework"],
    )
    .await?;
    Ok(())
}

/// Upgrade all modules
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn force_upgrade_mainnet_multiple() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;
    support::upgrade_multiple_impl(
        &mut s,
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
    )
    .await?;
    Ok(())
}
