mod support;

use libra_framework::release::ReleaseTarget;

/////// TEST ARBITRARY UPGRADES ///////
// do the same as above, but use the "arbitrary" upgrade policy to force an
// upgrade.
//
/// Force upgrade Libra
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn force_upgrade_mainnet_libra() {
    support::upgrade_multiple_impl(
        "upgrade-single-lib-force",
        vec!["1-libra-framework"],
        ReleaseTarget::Mainnet,
    )
    .await
    .unwrap();
}

/// Upgrade all modules
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn force_upgrade_mainnet_multiple() {
    support::upgrade_multiple_impl(
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
        ReleaseTarget::Mainnet,
    )
    .await
    .unwrap();
}
