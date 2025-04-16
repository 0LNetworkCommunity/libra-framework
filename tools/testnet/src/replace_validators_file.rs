use anyhow::anyhow;
use diem_types::{
    chain_id::{ChainId, NamedChain},
    transaction::{Transaction, WriteSetPayload},
};
use libra_config::validator_registration::ValCredentials;
use libra_rescue::cli_main::REPLACE_VALIDATORS_BLOB;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_types::core_types::app_cfg::{AppCfg, CONFIG_FILE_NAME};
use std::path::{Path, PathBuf};

use libra_rescue::{session_tools, transaction_factory};

// TODO: replace with calling the rescue_cli directly.
/// Make a rescue blob with the given credentials
/// credentials are usually saved by the libra-config tool
/// as a operator.yaml
pub async fn replace_validators_blob(
    db_path: &Path,
    creds: Vec<ValCredentials>,
    output_dir: &Path,
    upgrade_mrb_path: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    println!("run session to create validator onboarding tx (replace_validators_rescue.blob)");

    let cs = session_tools::register_and_replace_validators_changeset(
        db_path,
        creds,
        &upgrade_mrb_path,
        None, // intentionally forces chain_id=2 in simple swarm case
    )?;

    let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));
    let out = output_dir.join(REPLACE_VALIDATORS_BLOB);
    transaction_factory::save_rescue_blob(gen_tx, &out)?;
    Ok(out)
}

/// The libra-cli-config.yaml normally would have been initialized
/// with chain_id 4, testing, In swarm cases we want to see what the
/// chain_id actually is, since the a twin or rescue operation might change it
pub async fn set_chain_id_in_app_cfg(smoke: &mut LibraSmoke) -> anyhow::Result<()> {
    let client = smoke.client();
    let res = client.get_ledger_information().await?;
    let chain_id = res.inner().chain_id;
    let chain_name = NamedChain::from_chain_id(&ChainId::new(chain_id)).map_err(|e| anyhow!(e))?;

    for v in smoke.swarm.validators() {
        let cfg_path = v.config_path().parent().unwrap().join(CONFIG_FILE_NAME);
        let mut app_cfg = AppCfg::load(Some(cfg_path))?;
        let net = app_cfg.get_network_profile_mut(None)?;
        net.chain_name = chain_name;
        // only change this after the profile is updated
        app_cfg.workspace.default_chain_id = chain_name;

        app_cfg.save_file()?;
    }

    Ok(())
}
