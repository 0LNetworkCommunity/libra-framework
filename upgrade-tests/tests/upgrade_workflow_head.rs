mod support;

use libra_framework::release::ReleaseTarget;

/// Testing that we can upgrade the chain framework using txs tools.
/// NOTE: this aims to tests that the upgrade workflow works.
/// here we are applying an upgrade of a single file on a chain that uses the
/// same head.mrb release. To check if the upgrade is "compatible" with the
/// maiinet release, there see upgrade_compatible.

/// We assume a built transaction script for upgrade in
/// tests/fixtures/test_upgrade. If it is not there, there is a helper that will
/// refresh those fixtures once.

/// Workflow
/// 1. a validator can submit a proposal with txs
/// 2. the validator can vote for the proposal
/// 3. check that the proposal is resolvable
/// 4. resolve a propsosal by sending the upgrade payload.
/// 5. Check that the new function all_your_base can be called
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade_workflow_passes_on_stable_stdlib() {
    support::upgrade_multiple_impl(
        "upgrade-single-lib",
        vec!["1-move-stdlib"],
        ReleaseTarget::Head,
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// same as above but with multiple modules being upgraded
async fn smoke_upgrade_multiple_steps() {
    support::upgrade_multiple_impl(
        "upgrade-multi-lib",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
        ReleaseTarget::Head,
    )
    .await
    .unwrap();
}

/// do the same as above, but use the "arbitrary" upgrade policy to force an
/// upgrade.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade_multiple_steps_force() {
    support::upgrade_multiple_impl(
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
        ReleaseTarget::Head,
    )
    .await
    .unwrap();
}
