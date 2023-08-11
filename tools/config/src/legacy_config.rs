use anyhow::Context;
use libra_types::{
    exports::{AccountAddress, AuthenticationKey},
    legacy_types::{app_cfg::AppCfg, network_playlist::NetworkPlaylist},
};
use url::Url;
use std::path::PathBuf;
use zapatos_crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt};
use zapatos_types::chain_id::NamedChain;

pub async fn wizard(
    force_authkey: Option<AuthenticationKey>,
    force_address: Option<AccountAddress>,
    config_dir: Option<PathBuf>,
    chain_name: Option<NamedChain>,
    test_private_key: Option<String>,
    playlist_url: Option<Url>
) -> anyhow::Result<AppCfg> {
    let (authkey, address) = if force_authkey.is_some() && force_address.is_some() {
        (force_authkey.unwrap(), force_address.unwrap())
    } else if let Some(pk_string) = test_private_key {
        let pk = Ed25519PrivateKey::from_encoded_string(&pk_string)?;
        let account_keys = libra_wallet::account_keys::get_account_from_private(&pk);
        (account_keys.auth_key, account_keys.account)
    } else {
        let account_keys = libra_wallet::account_keys::get_keys_from_prompt()?.child_0_owner;
        (account_keys.auth_key, account_keys.account)
    };

    // if the user specified both a chain name and playlist, then the playlist will override the degault settings for the named chain.
    let np = if let Some(u) = playlist_url {
      NetworkPlaylist::from_url(u, chain_name).await.ok()
    } else {
      NetworkPlaylist::default_for_network(chain_name).await.ok()
    };

    let cfg = AppCfg::init_app_configs(authkey, address, config_dir, chain_name, np)?;

    let p = cfg.save_file().context(format!(
        "could not initialize configs at {}",
        cfg.workspace.node_home.to_str().unwrap()
    ))?;

    println!("Success, config saved to {}", p.to_str().unwrap());

    Ok(cfg)
}
