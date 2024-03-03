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
async fn smoke_upgrade_compatible() {
    support::upgrade_test_single_impl(
        "upgrade-multi-lib",
        "3-libra-framework",
        ReleaseTarget::Mainnet,
    )
    .await;
}

/// similar to above test, but we want to know if in the worst case
/// we could push a backward incompatible module.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn smoke_upgrade_can_force_arbitrary() {
    support::upgrade_test_single_impl(
        "upgrade-multi-lib-force", // "arbitrary" policy in metadata
        "3-libra-framework",
        ReleaseTarget::Mainnet,
    )
    .await;
}
