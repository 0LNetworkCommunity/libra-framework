use libra_rescue::test_support::setup_v7_reference_twin_db;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm;

/// Takes a known mainnet restore archive which has not received any writeset blocks
/// and restores it to a known state. Then, we try to drive it with random created accounts with the LibraSmoke testing
#[tokio::test]
async fn test_twin_smoke_from_v7_rescue() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;

    let mut smoke = LibraSmoke::new(Some(2), None).await?;

    twin_swarm::awake_frankenswarm(&mut smoke, Some(dir), None).await?;
    Ok(())
}
