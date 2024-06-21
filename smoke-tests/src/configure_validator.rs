use diem_forge::{LocalSwarm, Swarm};
use diem_sdk::crypto::PrivateKey;
use diem_sdk::types::LocalAccount;
use libra_types::core_types::network_playlist::NetworkPlaylist;
use libra_types::{exports::AuthenticationKey, core_types::app_cfg::AppCfg};
use std::path::PathBuf;
use url::Url;

/// Set up the 0L local files, and get an AppCfg back after initializing in a temp dir, that will drop at the end of the test.
pub async fn init_val_config_files(
    swarm: &mut LocalSwarm,
    nth: usize,
    dir: PathBuf,
) -> anyhow::Result<(LocalAccount, AppCfg)> {
    let info = swarm.diem_public_info_for_node(nth);
    let url: Url = info.url().parse().unwrap();

    let node = swarm.validators().next().unwrap();
    let np = NetworkPlaylist::new(Some(url), Some(diem_types::chain_id::NamedChain::TESTING));
    let mut app_cfg = AppCfg::init_app_configs(
        AuthenticationKey::ed25519(&node.account_private_key().as_ref().unwrap().public_key()),
        node.peer_id(),
        Some(dir),
        Some(np.chain_name),
        Some(np),
    )
    .unwrap();

    let pri_key = node
        .account_private_key()
        .as_ref()
        .expect("could not get pri_key")
        .private_key();
    let auth = AuthenticationKey::ed25519(&pri_key.public_key());
    let profile = app_cfg
        .get_profile_mut(None)
        .expect("could not get profile");
    profile.set_private_key(&pri_key);

    let local_account = LocalAccount::new(auth.derived_address(), pri_key, 0);

    Ok((local_account, app_cfg))
}
