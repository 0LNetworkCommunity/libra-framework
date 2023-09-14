use anyhow::Context;
use diem_crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt};
use diem_types::chain_id::NamedChain;
use libra_types::exports::Client;
use libra_types::type_extensions::client_ext::ClientExt;
use libra_types::{
    exports::{AccountAddress, AuthenticationKey},
    legacy_types::{app_cfg::AppCfg, network_playlist::NetworkPlaylist},
};
use libra_wallet::account_keys::{get_ol_legacy_address, AccountKeys};
use std::path::PathBuf;
use url::Url;

pub async fn wizard(
    force_authkey: Option<AuthenticationKey>,
    force_address: Option<AccountAddress>,
    config_dir: Option<PathBuf>,
    chain_name: Option<NamedChain>,
    test_private_key: Option<String>,
    playlist_url: Option<Url>,
    network_playlist: Option<NetworkPlaylist>,
) -> anyhow::Result<AppCfg> {
    let (authkey, mut address) = if force_authkey.is_some() && force_address.is_some() {
        (force_authkey.unwrap(), force_address.unwrap())
    } else if let Some(pk_string) = test_private_key {
        let pk = Ed25519PrivateKey::from_encoded_string(&pk_string)?;
        let account_keys = libra_wallet::account_keys::get_account_from_private(&pk);
        (account_keys.auth_key, account_keys.account)
    } else {
        let account_keys = prompt_for_account()?;

        (account_keys.auth_key, account_keys.account)
    };

    // if the user specified both a chain name and playlist, then the playlist will override the degault settings for the named chain.
    let mut np = match network_playlist {
        Some(a) => a,
        None => {
            if let Some(u) = playlist_url {
                NetworkPlaylist::from_url(u, chain_name).await
            } else {
                NetworkPlaylist::default_for_network(chain_name).await
            }?
        }
    };

    np.refresh_sync_status().await?;

    let url = np.pick_one()?;

    let client = Client::new(url);

    if client.get_index().await.is_ok() {
        address = client.lookup_originating_address(authkey).await?;
    }

    let cfg = AppCfg::init_app_configs(authkey, address, config_dir, chain_name, Some(np))?;

    let p = cfg.save_file().context(format!(
        "could not initialize configs at {}",
        cfg.workspace.node_home.to_str().unwrap()
    ))?;

    println!("Success, config saved to {}", p.to_str().unwrap());

    Ok(cfg)
}

/// Wrapper on get keys_from_prompt, which checks if this is a legacy account.
pub fn prompt_for_account() -> anyhow::Result<AccountKeys> {
    let mut account_keys = libra_wallet::account_keys::get_keys_from_prompt()?.child_0_owner;

    if dialoguer::Confirm::new()
        .with_prompt("Is this an OG founder account (pre-v7)?")
        .interact()?
    {
        account_keys.account = get_ol_legacy_address(account_keys.account)?;
    }

    Ok(account_keys)
}
