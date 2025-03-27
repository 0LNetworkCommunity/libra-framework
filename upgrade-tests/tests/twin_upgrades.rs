mod support;

use anyhow::Context;
use libra_framework::release::ReleaseTarget;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm::awake_frankenswarm;
// Here we are testing how a Twin modified swarm responds to an upgrade
// upgrades are being applied against Mainnet data which is recovered by a snapshot.

/// Meta test: Upgrade all modules on a no-op dummy database (same as swarm)
/// should have same behavior as a normal swarm.
#[ignore]
#[tokio::test]
async fn twin_test_all_upgrades_dummy() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(Some(1), None).await?;

    // Is not trying to restore from an actual Twin, hence None
    // just a meta integration test
    awake_frankenswarm(&mut smoke, None).await?;

    support::upgrade_multiple_impl(
        &mut smoke,
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn test_twin_swarm_noop() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(Some(1), None).await?;

    awake_frankenswarm(&mut smoke, None).await?;
    Ok(())
}

/// NOTE: WIP: depends on a restored DB having been created.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore]
async fn twin_test_stdlib_upgrade() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;

    // note: this DB needs to be a functioning restored snapshot
    let default_path = libra_types::global_config_dir();
    let p = default_path.join("data/db");
    assert!(p.exists());

    awake_frankenswarm(&mut s, Some(p)).await?;

    support::upgrade_multiple_impl(&mut s, "upgrade-single-lib", vec!["1-move-stdlib"]).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore]
async fn twin_test_framework_upgrade() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new_with_target(Some(1), None, ReleaseTarget::Mainnet)
        .await
        .context("could not start libra smoke")?;

    // note: this DB needs to be a functioning restored snapshot
    let default_path = libra_types::global_config_dir();
    let p = default_path.join("data/db");
    assert!(p.exists());

    awake_frankenswarm(&mut s, Some(p)).await?;

    support::upgrade_multiple_impl(
        &mut s,
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
    )
    .await?;
    Ok(())
}
