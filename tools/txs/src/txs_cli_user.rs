//! Validator subcommands

use crate::submit_transaction::Sender;
use diem::common::types::RotationProofChallenge;
use diem_sdk::crypto::ed25519::Ed25519PublicKey;
use diem_sdk::crypto::{PrivateKey, SigningKey, ValidCryptoMaterialStringExt};
use diem_sdk::types::LocalAccount;
use diem_types::account_address::AccountAddress;
use diem_types::{account_config::CORE_CODE_ADDRESS, transaction::TransactionPayload};
use libra_cached_packages::libra_stdlib;
use libra_types::{
    exports::{AuthenticationKey, Ed25519PrivateKey},
    type_extensions::client_ext::ClientExt,
};
use libra_wallet::account_keys::get_keys_from_prompt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum UserTxs {
    RotateKey(RotateKeyTx),
    RotationCapability(RotationCapabilityTx),
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
            UserTxs::RotationCapability(offer_rotation_capability) => {
                match offer_rotation_capability.run(sender).await {
                    Ok(_) => println!("SUCCESS: offered rotation capability"),
                    Err(e) => {
                        println!("ERROR: could not offer rotation capability, message: {}", e);
                    }
                }
            }
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
        let payload = libra_stdlib::slow_wallet_user_set_slow();
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct RotateKeyTx {
    #[clap(short, long)]
    /// The new authkey to be used
    pub new_private_key: Option<String>, // Dev NOTE: account address has the same bytes as AuthKey
    #[clap(short, long)]
    /// Account address for which rotation is done. It
    /// can be different from caller's address if rotation capability has been granted
    /// to the caller. Do not specify this if you want to rotate your own key.
    pub account_address: Option<String>,
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
        let payload = if let Some(account_address) = &self.account_address {
            let target_account_address = AccountAddress::from_str(account_address)?;
            let target_account = sender
                .client()
                .get_account(target_account_address)
                .await?
                .into_inner();
            // rotate key for account_address
            rotate_key_delegated(
                seq,
                &target_account_address, // account for which rotation is carried
                &target_account.authentication_key, // auth key for an account for which rotation is carried
                &new_private_key,
            )
        } else {
            // rotate key for self
            rotate_key(
                user_account,
                sender.local_account.private_key().to_owned(),
                sender.local_account.authentication_key(),
                seq,
                new_private_key,
            )
        }?;

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
    // form a rotation proof challenge. See account.move
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

/// Create the TransactionPayload for a delegated key transaction using rotation capability
pub fn rotate_key_delegated(
    sequence_number: u64,
    target_account_address: &AccountAddress, // account for which rotation is carried
    target_auth_key: &AuthenticationKey, // auth key for an account for which rotation is carried
    new_private_key: &Ed25519PrivateKey,
) -> anyhow::Result<TransactionPayload> {
    let new_public_key = Ed25519PublicKey::from(new_private_key);
    let rotation_proof = RotationProofChallenge {
        account_address: CORE_CODE_ADDRESS,
        module_name: String::from("account"),
        struct_name: String::from("RotationProofChallenge"),
        sequence_number,
        originator: *target_account_address,
        current_auth_key: AccountAddress::from_bytes(target_auth_key)?,
        new_public_key: new_public_key.to_bytes().to_vec(),
    };

    let rotation_msg = bcs::to_bytes(&rotation_proof)?;

    // Signs the struct using the next private key
    let rotation_proof_signed_by_new_private_key =
        new_private_key.sign_arbitrary_message(&rotation_msg);

    let payload = libra_stdlib::account_rotate_authentication_key_with_rotation_capability(
        *target_account_address,
        0,
        new_public_key.to_bytes().to_vec(),
        rotation_proof_signed_by_new_private_key.to_bytes().to_vec(),
    );

    Ok(payload)
}

#[derive(Serialize, Deserialize)]
pub struct RotationCapabilityOfferProofChallengeV2 {
    account_address: AccountAddress,
    module_name: String,
    struct_name: String,
    chain_id: u8,
    sequence_number: u64,
    source_address: AccountAddress,
    recipient_address: AccountAddress,
}

/// Offer rotation capability to a delegate address.
/// A delegate address now can rotate a key for this account owner
#[derive(clap::Args)]
pub struct RotationCapabilityTx {
    #[clap(short, long)]
    pub action: String,

    #[clap(short, long)]
    pub delegate_address: String,
}
impl RotationCapabilityTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let is_offer = match self.action.to_lowercase().as_str() {
            "offer" => true,
            "revoke" => false,
            _ => return Err(anyhow::anyhow!("Invalid action, allowed: offer, revoke")),
        };
        let user_account: AccountAddress = sender.local_account.address();
        let index_response = sender.client().get_index().await?;
        let chain_id = index_response.into_inner().chain_id;

        let recipient_address = AccountAddress::from_str(&self.delegate_address)?;
        let seq = sender.client().get_sequence_number(user_account).await?;
        let payload = if is_offer {
            offer_rotation_capability_v2(&sender.local_account, recipient_address, chain_id, seq)
        } else {
            revoke_rotation_capability(recipient_address)
        }?;

        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

pub fn offer_rotation_capability_v2(
    offerer_account: &LocalAccount,
    delegate_account: AccountAddress,
    chain_id: u8,
    sequence_number: u64,
) -> anyhow::Result<TransactionPayload> {
    let rotation_capability_offer_proof = RotationCapabilityOfferProofChallengeV2 {
        account_address: CORE_CODE_ADDRESS,
        module_name: String::from("account"),
        struct_name: String::from("RotationCapabilityOfferProofChallengeV2"),
        chain_id,
        sequence_number,
        source_address: offerer_account.address(),
        recipient_address: delegate_account,
    };

    let rotation_capability_proof_msg = bcs::to_bytes(&rotation_capability_offer_proof);
    let rotation_proof_signed = offerer_account
        .private_key()
        .clone()
        .sign_arbitrary_message(&rotation_capability_proof_msg.unwrap());

    let payload = libra_stdlib::account_offer_rotation_capability(
        rotation_proof_signed.to_bytes().to_vec(),
        0,
        offerer_account.public_key().to_bytes().to_vec(),
        delegate_account,
    );

    Ok(payload)
}

pub fn revoke_rotation_capability(
    delegate_account: AccountAddress,
) -> anyhow::Result<TransactionPayload> {
    let payload = libra_stdlib::account_revoke_rotation_capability(delegate_account);

    Ok(payload)
}
