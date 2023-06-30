
use libra_types::{legacy_types::app_cfg::AppCfg, exports::AuthenticationKey};
use libra_types::legacy_types::network_playlist::NetworkPlaylist;
use zapatos_sdk::types::LocalAccount;
use zapatos_forge::{Swarm, LocalSwarm};
use url::Url;
use zapatos_sdk::crypto::{PrivateKey, ValidCryptoMaterialStringExt};
use std::path::PathBuf;

/// Set up the 0L local files, and get an AppCfg back after initializing in a temp dir, that will drop at the end of the test.
pub async fn init_val_config_files(swarm: &mut LocalSwarm, nth: usize, dir: PathBuf) -> anyhow::Result<(LocalAccount, AppCfg)> {

    let info = swarm.aptos_public_info_for_node(nth);
    let url: Url = info.url().parse().unwrap();

    let node = swarm.validators().into_iter().next().unwrap();
    let np = NetworkPlaylist::testing(Some(url));
    let mut app_cfg = AppCfg::init_app_configs(
      AuthenticationKey::ed25519(&node.account_private_key().as_ref().unwrap().public_key()),
      node.peer_id(),
      Some(dir),
      Some(np.chain_id),
      Some(np),
    ).unwrap();

    let pri_key = node.account_private_key().as_ref().expect("could not get pri_key").private_key();
    let auth = AuthenticationKey::ed25519(&pri_key.public_key());
    let profile = app_cfg.get_profile_mut(None).expect("could not get profile");
    profile.test_private_key = Some(pri_key.clone());
    // dbg!(&profile);


    let local_account = LocalAccount::new(auth.derived_address(), pri_key, 0);

  Ok((local_account, app_cfg))
}