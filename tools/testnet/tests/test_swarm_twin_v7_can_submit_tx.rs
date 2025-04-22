use libra_framework::release::ReleaseTarget;
use libra_rescue::test_support::setup_v7_reference_twin_db;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_testnet::twin_swarm;
use libra_txs::{
    txs_cli::{TxsCli, TxsSub},
    txs_cli_user::UserTxs::HumanFounder,
};
use libra_types::core_types::app_cfg::AppCfg;

/// Checks we can submit the minimal transaction to the twin network
/// after a twin network is set
#[tokio::test]
async fn test_swarm_twin_v7_upgrade_tx_success() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;

    let mut smoke = LibraSmoke::new(Some(2), None).await?;

    let head_mrb_path = ReleaseTarget::Head.find_bundle_path()?;

    // use head.mrb as the reference mrb
    let test_info =
        twin_swarm::awake_frankenswarm(&mut smoke, Some(dir), Some(head_mrb_path)).await?;

    // checks if 0x1::all_your_base module is present
    // should use api to list the modules installed on 0x1
    // let client = smoke.client();
    let val_one = &test_info[0];

    let cfg = AppCfg::load(Some(val_one.app_cfg_path.clone()))?;
    dbg!(&cfg);

    let txs_cli = TxsCli {
        subcommand: Some(TxsSub::User(HumanFounder)),
        config_path: Some(val_one.app_cfg_path.clone()),
        ..Default::default()
    };

    txs_cli.run().await?;

    Ok(())
}
