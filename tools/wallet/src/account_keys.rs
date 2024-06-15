//! Use ol-keys to generate or parse keys using the legacy key derivation scheme
use crate::{
    core::{legacy_scheme::LegacyKeyScheme, wallet_library::WalletLibrary},
    key_gen::keygen,
    load_keys,
};

use anyhow::Result;
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_crypto::PrivateKey;
use diem_types::account_address::AccountAddress;
use diem_types::transaction::authenticator::AuthenticationKey;
use serde::Serialize;
use std::path::Path;
use std::str::FromStr;

#[derive(Serialize)]
/// A Struct to store ALL the legacy keys for storage.
pub struct KeyChain {
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

/// The AccountAddress and AuthenticationKey are diem structs, they have the same NAME in the diem_types crate. So we need to cast them into usuable structs.
#[derive(Serialize)]
pub struct AccountKeys {
    // TODO: change this to use vendor AccountKey
    /// The account address derived from AuthenticationKey
    pub account: AccountAddress,
    /// The authentication key derived from private key
    pub auth_key: AuthenticationKey,
    /// The private key
    pub pri_key: Ed25519PrivateKey,
}

//////// 0L ////////
/// addresses in the 0L chains before V7 had a truncated address of 16 bytes
pub fn get_ol_legacy_address(addr: AccountAddress) -> anyhow::Result<AccountAddress> {
    // keep only last 16 bytes

    let leg_addr = hex::encode(&addr[16..]);

    let literal = &format!("0x0000000000000000{}", leg_addr);

    Ok(AccountAddress::from_hex_literal(literal)?)
}

/// Derive keys from a mnemonic in the 0L scheme
// NOTE: these keys are not sufficient to create a validator from V7 onwards. There are BLS keys needed in additiont o Ed25519
pub fn legacy_keygen(danger_print: bool) -> Result<KeyChain> {
    let (auth_key, account, wallet, mnem) = keygen();

    //////////////// Info ////////////////

    if danger_print {
        println!(
            "0L Account Address:\n\
          ...........................\n\
          {}\n",
            &account.to_string()
        );

        println!(
            "Authentication Key (for key rotation):\n\
          ...........................\n\
          {}\n",
            &auth_key.to_string()
        );

        println!(
            "0L mnemonic:\n\
          ..........................."
        );

        //use same styles as abscissa_info
        println!("\x1b[1;36m{}\n\x1b[0m", mnem.as_str());

        println!(
            "WRITE THIS DOWN NOW. This is the last time you will see \
                    this mnemonic. It is not saved anywhere. Nobody can help \
                    you if you lose it.\n\n"
        );
    }

    KeyChain::new(&wallet)
}

/// Get the legacy keys from the wallet
pub fn get_keys_from_prompt() -> Result<KeyChain> {
    let (_auth_key, _account, wallet) = load_keys::get_account_from_prompt();
    KeyChain::new(&wallet)
}

/// for libs to get the keys from a mnemonic
pub fn get_keys_from_mnem(mnem: String) -> Result<KeyChain> {
    let (_auth_key, _account, wallet) = load_keys::get_account_from_mnem(mnem)?;
    KeyChain::new(&wallet)
}

pub fn get_account_from_private(pri_key: &Ed25519PrivateKey) -> AccountKeys {
    let pub_key = pri_key.public_key();
    let auth_key = AuthenticationKey::ed25519(&pub_key);
    let account = auth_key.derived_address();

    AccountKeys {
        account,
        auth_key,
        pri_key: pri_key.clone(),
    }
}

fn get_account_from_nth(w: &WalletLibrary, n: u8) -> Result<AccountKeys> {
    let pri_keys = LegacyKeyScheme::new(w);

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

impl KeyChain {
    pub fn new(w: &WalletLibrary) -> Result<Self> {
        Ok(KeyChain {
            mnemonic: w.mnemonic(),
            seed: w.get_key_factory().main().to_owned(),
            child_0_owner: get_account_from_nth(w, 0)?,
            child_1_operator: get_account_from_nth(w, 1)?,
            child_2_val_network: get_account_from_nth(w, 2)?,
            child_3_fullnode_network: get_account_from_nth(w, 3)?,
            child_4_consensus: get_account_from_nth(w, 4)?,
            child_5_executor: get_account_from_nth(w, 5)?,
        })
    }

    /// Save the legacy keys to a json file
    pub fn save_keys(&self, dir: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let path = dir.join("legacy_keys.json");
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn display(&self, display_private: bool) {
        if display_private {
            eprintln!("{}", serde_json::to_string_pretty(&self).unwrap());
        } else {
            let owner = &self.child_0_owner;
            println!("owner account: {}", owner.account);
            // TODO: include more keys to derive
        }
    }
}

#[test]
fn test_legacy_keys() {
    let alice_mnem = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse";

    let l = get_keys_from_mnem(alice_mnem.to_string()).unwrap();

    assert_eq!(get_ol_legacy_address(l.child_0_owner.account)
                   .unwrap()
                   .to_string(), "000000000000000000000000000000004c613c2f4b1e67ca8d98a542ee3f59f5");

    assert_eq!("2570472a9a08b9cc1f7c616e9ebb1dc534db452d3a3d3c567e58bec9f0fbd13e", &hex::encode(&l.seed));
}

#[test]
// We want to check that the address and auth key derivation is the same from what Diem generates, and what the vendor types do.
fn type_conversion_give_same_auth_and_address() {
    use crate::load_keys::get_account_from_mnem;

    let alice_mnem = "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse";

    let (auth_key, account, wallet) = get_account_from_mnem(alice_mnem.to_owned()).unwrap();

    let l = KeyChain::new(&wallet).unwrap();

    assert_eq!(account.to_hex_literal(), l.child_0_owner.account.to_hex_literal());
    assert_eq!(auth_key.to_string(), l.child_0_owner.auth_key.to_string());

    // Check the vendor ConfigKey struct is the same.
    use diem_config::keys::ConfigKey;
    use diem_crypto::ed25519::Ed25519PrivateKey;

    let cfg_key: ConfigKey<Ed25519PrivateKey> = ConfigKey::new(l.child_0_owner.pri_key);
    let auth_key_from_cfg = AuthenticationKey::ed25519(&cfg_key.public_key()).derived_address();
    assert_eq!(auth_key_from_cfg.to_string(), l.child_0_owner.auth_key.to_string());
}
