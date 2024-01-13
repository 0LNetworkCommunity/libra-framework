//! Validator subcommands

use crate::submit_transaction::Sender;
use diem::common::types::RotationProofChallenge;
use diem_sdk::crypto::{PrivateKey, SigningKey, ValidCryptoMaterialStringExt};
use diem_types::{
    account_address::AccountAddress, account_config::CORE_CODE_ADDRESS,
    transaction::TransactionPayload,
};
use libra_cached_packages::libra_stdlib;
use libra_types::{
    exports::{AuthenticationKey, Ed25519PrivateKey},
    type_extensions::client_ext::ClientExt,
};
use libra_wallet::account_keys::get_keys_from_prompt;

#[derive(clap::Subcommand)]
pub enum UserTxs {
    RotateKey(RotateKeyTx),
    SetSlow(SetSlowTx),
}

impl UserTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        match &self {
            UserTxs::RotateKey(rotate) => match rotate.run(sender).await {
                Ok(_) => println!("SUCCESS: private key rotated"),
                Err(e) => {
                    println!("ERROR: could not rotate private key, message: {}", e);
                }
            },
            UserTxs::SetSlow(slow) => match slow.run(sender).await {
                Ok(_) => println!("SUCCESS: account set to Slow Wallet"),
                Err(e) => {
                    println!(
                        "ERROR: could set the account to Slow Wallet, message: {}",
                        e
                    );
                }
            },
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct SetSlowTx {
    // TODO: any arguments needed? Confirmation?
}

impl SetSlowTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::slow_wallet_set_slow();
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct RotateKeyTx {
    #[clap(short, long)]
    /// The new authkey to be used
    new_private_key: Option<String>, // Dev NOTE: account address has the same bytes as AuthKey
}

impl RotateKeyTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let user_account: AccountAddress = sender.local_account.address();

        let new_private_key = if let Some(pk) = &self.new_private_key {
            Ed25519PrivateKey::from_encoded_string(pk)?
        } else {
            let legacy = get_keys_from_prompt()?;
            legacy.child_0_owner.pri_key
        };

        let seq = sender.client().get_sequence_number(user_account).await?;
        let payload = rotate_key(
            user_account,
            sender.local_account.private_key().to_owned(),
            sender.local_account.authentication_key(),
            seq,
            new_private_key,
        )?;

        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

/// create the TransactionPayload for a key rotation (a signed rotation challenge)
pub fn rotate_key(
    sender_address: AccountAddress,
    current_private_key: Ed25519PrivateKey,
    auth_key: AuthenticationKey,
    sequence_number: u64,
    new_private_key: Ed25519PrivateKey,
) -> anyhow::Result<TransactionPayload> {
    // form a rotation proof challence. See account.move
    let rotation_proof = RotationProofChallenge {
        account_address: CORE_CODE_ADDRESS,
        module_name: "account".to_string(),
        struct_name: "RotationProofChallenge".to_string(),
        sequence_number,
        originator: sender_address,
        current_auth_key: AccountAddress::from_bytes(auth_key)?,
        new_public_key: new_private_key.public_key().to_bytes().to_vec(),
    };

    // get the bytes of the challenge
    let rotation_msg = bcs::to_bytes(&rotation_proof)?;

    // Signs the struct using both the current private key and the next private key
    let rotation_proof_signed_by_current_private_key =
        current_private_key.sign_arbitrary_message(&rotation_msg);
    let rotation_proof_signed_by_new_private_key =
        new_private_key.sign_arbitrary_message(&rotation_msg);

    let payload = libra_stdlib::account_rotate_authentication_key(
        0,
        // Existing public key
        current_private_key.public_key().to_bytes().to_vec(),
        0,
        // New public key
        new_private_key.public_key().to_bytes().to_vec(),
        rotation_proof_signed_by_current_private_key
            .to_bytes()
            .to_vec(),
        rotation_proof_signed_by_new_private_key.to_bytes().to_vec(),
    );

    Ok(payload)
}
