use crate::{
    keys::{refresh_validator_files, validator_keygen},
    load_keys,
    core::wallet_library::WalletLibrary,
};
use anyhow::Result;
use indoc::formatdoc;
// use ol_keys::wallet::get_account_from_mnem;
use libra_types::exports::{AccountAddress, AuthenticationKey};
use std::path::PathBuf;
use zapatos_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};

/// Genereates keys from WalletLibrary, updates a MinerConfig
pub fn keygen() -> (AuthenticationKey, AccountAddress, WalletLibrary, String) {
    // Generate new keys
    let mut wallet = WalletLibrary::new();
    let mnemonic_string = wallet.mnemonic();
    // NOTE: Authkey uses the child number 0 by default
    let (auth_key, _) = wallet.new_address().expect("Could not generate address");
    let account = auth_key.derived_address();
    //////////////// Info ////////////////

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
    println!("\x1b[1;36m{}\n\x1b[0m", &mnemonic_string.as_str());

    println!(
        "WRITE THIS DOWN NOW. This is the last time you will see \
                  this mnemonic. It is not saved anywhere. Nobody can help \
                  you if you lose it.\n\n"
    );

    (auth_key, account, wallet, mnemonic_string)
}

pub async fn run(mnemonic: Option<String>, output_dir: Option<PathBuf>) -> Result<String> {
    let private_key = if let Some(mnemonic) = mnemonic {
        let (_, account_address, wallet_lib) = load_keys::get_account_from_mnem(mnemonic.clone())?;

        refresh_validator_files(Some(mnemonic), output_dir, false)?;

        Ed25519PrivateKey::try_from(
            wallet_lib
                .get_private_key(&account_address)?
                .to_bytes()
                .as_ref(),
        )?
    } else {
        let (_, _, private_identity, _) = validator_keygen(output_dir)?;
        private_identity.account_private_key
    };

    let public_key = Ed25519PublicKey::from(&private_key);
    let authentication_key = AuthenticationKey::ed25519(&public_key);
    let private_key = hex::encode(private_key.to_bytes());
    let account_address = authentication_key.derived_address().to_hex_literal();

    Ok(formatdoc!(
        r#"
            ====================================
            Private key: {private_key}
            Public key: {public_key}
            Authentication key: {authentication_key}
            Account address: {account_address}
        "#
    ))
}

#[cfg(test)]
mod tests {
    use super::run;
    use anyhow::{bail, Result};
    use std::{fs, path::PathBuf};

    const ALICE_MNEMONIC: &str =
        "talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse";
    #[tokio::test]
    async fn save_val_keys_from_mnemonic() -> Result<()> {
        let this_dir: PathBuf = env!("CARGO_MANIFEST_DIR").parse()?;
        let output_dir = this_dir.join("temp_two");
        let result = run(Some(ALICE_MNEMONIC.to_string()), Some(output_dir.clone()))
            .await
            .unwrap();

        let result = result.split("\n").collect::<Vec<_>>();

        let private_key = hex::decode(result[1].replace("Private key: ", "")).unwrap();
        assert_eq!(32, private_key.len());
        let public_key = hex::decode(result[2].replace("Public key: ", "")).unwrap();
        assert_eq!(32, public_key.len());
        let authentication_key =
            hex::decode(result[3].replace("Authentication key: ", "")).unwrap();
        assert_eq!(32, authentication_key.len());
        assert!(result[4].starts_with("Account address: 0x"));

        // Ensure yaml files exist
        for yaml_file in [
            "private-keys.yaml",
            "public-keys.yaml",
            "validator-full-node-identity.yaml",
            "validator-identity.yaml",
        ] {
            let path = output_dir.join(yaml_file);
            if !fs::metadata(&path).is_ok() {
                // Clean up
                // fs::remove_dir_all(output_dir).ok();
                // Stop the test with error
                bail!("File does not exist: {path:?}");
            }
        }

        // Clean up
        // fs::remove_dir_all(output_dir).ok();
        Ok(())
    }

    #[tokio::test]
    async fn keygen_with_none_mnem() -> Result<()> {
        let this_dir: PathBuf = env!("CARGO_MANIFEST_DIR").parse()?;

        let output_dir = this_dir.join("temp");
        let result = run(None, Some(output_dir.clone())).await.unwrap();
        let result = result.split("\n").collect::<Vec<_>>();

        let private_key = hex::decode(result[1].replace("Private key: ", "")).unwrap();
        assert_eq!(32, private_key.len());
        let public_key = hex::decode(result[2].replace("Public key: ", "")).unwrap();
        assert_eq!(32, public_key.len());
        let authentication_key =
            hex::decode(result[3].replace("Authentication key: ", "")).unwrap();
        assert_eq!(32, authentication_key.len());
        assert!(result[4].starts_with("Account address: 0x"));

        // Ensure yaml files exist
        for yaml_file in [
            "private-keys.yaml",
            "public-keys.yaml",
            "validator-full-node-identity.yaml",
            "validator-identity.yaml",
        ] {
            let path = output_dir.join(yaml_file);
            if !fs::metadata(&path).is_ok() {
                // Clean up
                fs::remove_dir_all(output_dir).ok();
                // Stop the test with error
                bail!("File does not exist: {path:?}");
            }
        }

        // Clean up
        fs::remove_dir_all(output_dir).ok();
        Ok(())
    }
}
