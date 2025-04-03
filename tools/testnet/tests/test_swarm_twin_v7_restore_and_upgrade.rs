use libra_framework::release::ReleaseTarget;
use libra_rescue::test_support::setup_v7_reference_twin_db;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm;

/// Takes a known mainnet restore archive which has not received any writeset blocks
/// and restores it to a known state. Then, we try to drive it with random created accounts with the LibraSmoke testing
#[tokio::test]
async fn test_twin_smoke_from_v7_rescue_and_upgrade() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;

    let mut smoke = LibraSmoke::new(Some(2), None).await?;
    let framework_mrb_path = ReleaseTarget::Head.find_bundle_path().ok();
    twin_swarm::awake_frankenswarm(&mut smoke, Some(dir), framework_mrb_path).await?;

    // checks if 0x1::founder module is present
    // should use api to list the modules installed on 0x1
    let client = smoke.client();
    let modules = client.get_account_modules("0x1".parse()?).await?;

    let exists = modules.inner().iter().any(|module| {
        let new_m = module
            .clone()
            .try_parse_abi()
            .expect("failed to parse module");
        if let Some(m) = &new_m.abi {
            return m.name.to_string().contains("libra_coin");
        }
        false
    });
    println!("module exists: {}", exists);
    Ok(())
}
