mod support;

use libra_rescue::test_support::setup_test_db;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm::awake_frankenswarm;

// The holy grail of e2e tests, or the final resting place of the heroes who tried
//   ___  _                                ______        _
//  / (_)| |           o                  (_) |  o      | |    |
//  \__  | |        ,      __,   _  _        _|_     _  | |  __|   ,
//  /    |/  |   | / \_|  /  |  / |/ |      / | ||  |/  |/  /  |  / \_
//  \___/|__/ \_/|/ \/ |_/\_/|_/  |  |_/   (_/   |_/|__/|__/\_/|_/ \/
//              /|
//              \|
// Here we are testing how a Twin modified swarm responds to an upgrade
// upgrades are being applied against Mainnet data which is recovered by a snapshot.

#[ignore]
#[tokio::test]
/// Should be able to take a production db (twin)
/// and the new validators upgrade with current HEAD Move code
async fn twin_test_head_upgrade() -> anyhow::Result<()> {
    let dir = setup_test_db()?;
    let mut smoke = LibraSmoke::new(Some(2), None).await?;

    // Is not trying to restore from an actual Twin, hence None
    // just a meta integration test
    awake_frankenswarm(&mut smoke, Some(dir)).await?;

    support::upgrade_multiple_impl(
        &mut smoke,
        "upgrade-multi-lib-force",
        vec!["1-move-stdlib", "2-vendor-stdlib", "3-libra-framework"],
    )
    .await?;
    Ok(())
}
