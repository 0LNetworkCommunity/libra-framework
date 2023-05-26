//! Use ol-keys to generate or parse keys using the legacy key derivation scheme

use anyhow::Result;
use diem_wallet::WalletLibrary;
use ol_keys::wallet::{get_account_from_mnem, keygen};
use ol_keys::{scheme::KeyScheme, wallet::get_account_from_prompt};
use serde::Serialize;
use std::path::Path;
use std::str::FromStr;
use zapatos_crypto::ed25519::Ed25519PrivateKey;
use zapatos_types::account_address::AccountAddress;
use zapatos_types::transaction::authenticator::AuthenticationKey;

#[derive(Serialize)]
/// A Struct to store ALL the legacy keys for storage.
pub struct LegacyKeys {
    /// The mnemonic
    pub mnemonic: String,
    /// seed generated from mnemonic
    pub seed: Vec<u8>,
    /// The main account address
    pub child_0_owner: AccountKeys,
    /// The operator account address
    pub child_1_operator: AccountKeys,
    /// The validator network identity
    pub child_2_val_network: AccountKeys,
    /// The fullnode network identity
    pub child_3_fullnode_network: AccountKeys,
    /// The consensus key
    pub child_4_consensus: AccountKeys,
    /// The execution key
    pub child_5_executor: AccountKeys,
}

/// The AccountAddress and AuthenticationKey are zapatos structs, they have the same NAME in the diem_types crate. So we need to cast them into usuable structs.
#[derive(Serialize)]
pub struct AccountKeys {
    /// The account address derived from AuthenticationKey
    pub account: AccountAddress,
    /// The authentication key derived from private key
    pub auth_key: AuthenticationKey,
    /// The private key
    pub pri_key: Ed25519PrivateKey,
}

/// Legacy Keygen. These note these keys are not sufficient to create a validator from V7 onwards. Besides the Mnemonic the keypair for 0th derivation (owner key) is reusable.
pub fn legacy_keygen() -> Result<LegacyKeys> {
    let (_auth_key, _account, wallet, _mnem) = keygen();
    LegacyKeys::new(&wallet)
}

/// Get the legacy keys from the wallet
pub fn get_keys_from_prompt() -> Result<LegacyKeys> {
    let (_auth_key, _account, wallet) = get_account_from_prompt();
    LegacyKeys::new(&wallet)
}

/// for libs to get the keys from a mnemonic
pub fn get_keys_from_mnem(mnem: String) -> Result<LegacyKeys> {
    let (_auth_key, _account, wallet) = get_account_from_mnem(mnem)?;
    LegacyKeys::new(&wallet)
}

fn get_account_from_private_key(w: &WalletLibrary, n: u8) -> Result<AccountKeys> {
    let pri_keys = KeyScheme::new(w);

    let key = match n {
        0 => pri_keys.child_0_owner,
        1 => pri_keys.child_1_operator,
        2 => pri_keys.child_2_val_network,
        3 => pri_keys.child_3_fullnode_network,
        4 => pri_keys.child_4_consensus,
        5 => pri_keys.child_5_executor,
        _ => panic!("Invalid key index"),
    };

    let auth_key = key.get_authentication_key();
    let account = key.get_address();
    Ok(AccountKeys {
        account: AccountAddress::from_hex_literal(&account.to_hex_literal())?,
        auth_key: AuthenticationKey::from_str(&auth_key.to_string())?,
        pri_key: Ed25519PrivateKey::try_from(key.get_private_key().to_bytes().as_ref())?,
    })
}

impl LegacyKeys {
    pub fn new(w: &WalletLibrary) -> Result<Self> {
        Ok(LegacyKeys {
            mnemonic: w.mnemonic(),
            seed: w.get_key_factory().main().to_owned(),
            child_0_owner: get_account_from_private_key(w, 0)?,
            child_1_operator: get_account_from_private_key(w, 1)?,
            child_2_val_network: get_account_from_private_key(w, 2)?,
            child_3_fullnode_network: get_account_from_private_key(w, 3)?,
            child_4_consensus: get_account_from_private_key(w, 4)?,
            child_5_executor: get_account_from_private_key(w, 5)?,
        })
    }

    /// Save the legacy keys to a json file
    pub fn save_keys(&self, dir: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let path = dir.join("legacy_keys.json");
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn display(&self) {
        eprintln!("{}", serde_json::to_string_pretty(&self).unwrap());
    }
}

#[test]
fn test_legacy_keys() {
    let alice_mnem = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse";

    let l = get_keys_from_mnem(alice_mnem.to_string()).unwrap();

    assert!(
        &l.child_0_owner.account.to_string()
            == "000000000000000000000000000000004c613c2f4b1e67ca8d98a542ee3f59f5"
    );

    assert!(
        "2570472a9a08b9cc1f7c616e9ebb1dc534db452d3a3d3c567e58bec9f0fbd13e" == &hex::encode(&l.seed)
    );
}

#[test]
// We want to check that the address and auth key derivation is the same from what Diem generates, and what the vendor types do.
fn type_conversion_give_same_auth_and_address() {
    let alice_mnem = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse";

    let (auth_key, account, wallet) = get_account_from_mnem(alice_mnem.to_owned()).unwrap();

    let l = LegacyKeys::new(&wallet).unwrap();

    assert!(account.to_hex_literal() == l.child_0_owner.account.to_hex_literal());
    assert!(auth_key.to_string() == l.child_0_owner.auth_key.to_string());

    // Check the vendor ConfigKey struct is the same.
    use zapatos_config::keys::ConfigKey;
    use zapatos_crypto::ed25519::Ed25519PrivateKey;

    let cfg_key: ConfigKey<Ed25519PrivateKey> = ConfigKey::new(l.child_0_owner.pri_key);
    let auth_key_from_cfg = AuthenticationKey::ed25519(&cfg_key.public_key()).derived_address();
    assert!(auth_key_from_cfg.to_string() == l.child_0_owner.auth_key.to_string());
}
