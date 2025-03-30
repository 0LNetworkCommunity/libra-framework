use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm;

/// Start a swarm, to produce some db artifacts
/// and use those artifacts to create a twin swarm.
/// It's effectively a noop, that just tests the tooling.
#[tokio::test]
async fn test_twin_swarm_noop() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(Some(2), None).await?;

    twin_swarm::awake_frankenswarm(&mut smoke, None).await?;
    Ok(())
}
