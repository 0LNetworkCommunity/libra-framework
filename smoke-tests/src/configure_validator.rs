use diem_forge::{LocalSwarm, Node};
use diem_sdk::crypto::PrivateKey;
use diem_sdk::types::LocalAccount;
use diem_types::chain_id::NamedChain;
use libra_types::core_types::network_playlist::NetworkPlaylist;
use libra_types::{core_types::app_cfg::AppCfg, exports::AuthenticationKey};
use std::path::PathBuf;

/// Set up the 0L local files, and get an AppCfg back after initializing in a temp dir, that will drop at the end of the test.
pub fn init_val_config_files(
    swarm: &mut LocalSwarm,
    nth: usize,
    dir_opt: Option<PathBuf>,
) -> anyhow::Result<(LocalAccount, AppCfg)> {
    // TODO: unclear why public info needs to be a mutable borrow
    let node = swarm
        .validators()
        .nth(nth)
        .expect("could not get nth validator");
    let url = node.rest_api_endpoint();

    let dir = dir_opt.unwrap_or(node.config_path().parent().unwrap().to_owned());

    let chain_name = NamedChain::from_chain_id(&swarm.chain_id()).ok();
    let np = NetworkPlaylist::new(Some(url), chain_name);
    let cfg_key = node.account_private_key().as_ref().unwrap();
    let prikey = cfg_key.private_key();
    let pubkey = prikey.public_key();
    let mut app_cfg = AppCfg::init_app_configs(
        AuthenticationKey::ed25519(&pubkey),
        node.peer_id(),
        Some(dir),
        Some(np.chain_name),
        Some(np),
    )?;

    let profile = app_cfg
        .get_profile_mut(None)
        .expect("could not get profile");

    profile.set_private_key(&prikey);

    let local_account = LocalAccount::new(profile.account, prikey, 0);

    Ok((local_account, app_cfg))
}

/// helper to save libra-cli config files for each of the validators in
/// their local temp folder (alongside validator.yaml)
pub fn save_cli_config_all(swarm: &mut LocalSwarm) -> anyhow::Result<()> {
    let len = swarm.validators().count();
    for i in 0..len {
        // a libra-cli-config file will be created at the temp swarm
        // directory of the node
        let (_, app_cfg) = init_val_config_files(swarm, i, None)?;
        let _file = app_cfg.save_file()?;
    }
    Ok(())
}
