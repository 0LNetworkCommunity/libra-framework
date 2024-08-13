mod support;

use anyhow::Context;
use libra_framework::release::ReleaseTarget;
use libra_smoke_tests::libra_smoke::LibraSmoke;

// Here we are testing if the Move source is actually compatible with prior
// mainnet release. (We assume the TX tools for upgrade flow work)
// Starting from previous mainnet.mrb, we'll try to upgrade with the current
// fixtures.
// NOTE: There are other tests here concerned with the TX workflow for
// upgrades.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn compatible_upgrade_mainnet_libra() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;
    support::upgrade_multiple_impl(&mut s, "upgrade-multi-lib", vec!["3-libra-framework"]).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// same as above but with multiple modules being upgraded
async fn compatible_upgrade_mainnet_multiple() -> anyhow::Result<()>{
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;

    support::upgrade_multiple_impl(
        &mut s,
        "upgrade-multi-lib",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
    )
    .await?;
  Ok(())
}
