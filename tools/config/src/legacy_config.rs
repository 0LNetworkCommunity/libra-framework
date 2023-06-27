use libra_types::legacy_types::app_cfg::AppCfg;
use libra_types::exports::AuthenticationKey;
use libra_types::exports::AccountAddress;
use zapatos_crypto::{ValidCryptoMaterialStringExt, ed25519::Ed25519PrivateKey};
use zapatos_types::chain_id::NamedChain;
use std::path::PathBuf;

pub fn wizard(
  force_authkey: Option<AuthenticationKey>,
  force_address: Option<AccountAddress>,
  config_dir: Option<PathBuf>,
  chain_name: Option<NamedChain>,
  test_private_key: Option<String>,
) -> anyhow::Result<AppCfg> {
  let (authkey, address) = if force_authkey.is_some() && force_address.is_some() {
    (force_authkey.unwrap(), force_address.unwrap())
  } else if let Some(pk_string) = test_private_key {
    let pk = Ed25519PrivateKey::from_encoded_string(&pk_string)?;
    let account_keys = libra_wallet::legacy::get_account_from_private(&pk);
    (account_keys.auth_key, account_keys.account)
  } else {
    let account_keys = libra_wallet::legacy::get_keys_from_prompt()?.child_0_owner;
    (account_keys.auth_key, account_keys.account)
  };

  let cfg = AppCfg::init_app_configs(authkey, address, config_dir, chain_name)?;

  cfg.save_file()?;

  Ok(cfg)
}