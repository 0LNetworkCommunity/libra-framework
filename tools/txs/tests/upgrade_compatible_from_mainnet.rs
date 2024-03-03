mod support;


use std::fs;
use diem_types::chain_id::NamedChain;
use libra_framework::{release::ReleaseTarget, upgrade_fixtures};
use libra_query::query_view;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};
use libra_txs::{
    txs_cli::{TxsCli, TxsSub::Governance},
    txs_cli_governance::GovernanceTxs::{Propose, Resolve, Vote},
};
use libra_types::legacy_types::app_cfg::TxCost;


/// Here we are testing if the Move source is actually compatible with prior
/// mainnet release. The other tests here are concerned with the TX workflow for
/// upgrades, using generally benign changes (adding a all_your_base.move noop
/// module).
/// starting from previous mainnet.mrb, we'll try to upgrade with the current
/// fixtures.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade_mainnet_compatible_libra() {
    support::upgrade_test_single_impl(
        "upgrade-multi-lib",
        "3-libra-framework",
        ReleaseTarget::Mainnet,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// same as above but with multiple modules being upgraded
async fn smoke_upgrade_mainnet_compatible_multiple() {
    support::upgrade_multiple_impl(
        "upgrade-multi-lib",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
        ReleaseTarget::Mainnet,
    )
    .await;
}

/// do the same as above, but use the "arbitrary" upgrade policy to force an
/// upgrade.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade_mainnet_compatible_multiple_force() {
    support::upgrade_multiple_impl(
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
        ReleaseTarget::Mainnet,
    )
    .await;
}
