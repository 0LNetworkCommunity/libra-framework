use crate::submit_transaction::Sender as LibraSender;
use diem_logger::error;
use diem_logger::info;
use diem_sdk::crypto::ed25519::Ed25519Signature;
use diem_sdk::crypto::HashValue;
use diem_sdk::crypto::Signature;
use diem_sdk::crypto::SigningKey;
use diem_sdk::types::LocalAccount;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use libra_query::chain_queries;
use libra_types::exports::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(clap::Args, Debug)]
pub struct PofBidArgs {
    #[clap(short, long)]
    /// bid amount for validator net reward
    pub net_reward: u64,
    #[clap(short, long)]
    /// seconds to wait between retries
    pub delay: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PofBidData {
    entry_fee: u64,
    epoch: u64,
}

#[allow(dead_code)]
/// these are the bytes that will be encoded with BCS
/// and then signed with ed25519 key
/// See the move equivalent in secret_bid.move
impl PofBidData {
    fn new(entry_fee: u64) -> Self {
        PofBidData {
            entry_fee,
            epoch: 0,
        }
    }
    fn to_bcs(&self) -> anyhow::Result<Vec<u8>> {
        Ok(bcs::to_bytes(&self)?)
    }

    fn from_bcs(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }

    async fn update_epoch(&mut self, client: &Client) -> anyhow::Result<()> {
        let num = chain_queries::get_epoch(client).await?;
        self.epoch = num;
        Ok(())
    }

    fn sign_bcs_bytes(&self, keys: &LocalAccount) -> anyhow::Result<Ed25519Signature> {
        let bcs = self.to_bcs()?;
        let signed = keys.private_key().sign_arbitrary_message(&bcs);
        assert!(
            signed.verify_arbitrary_msg(&bcs, keys.public_key()).is_ok(),
            "cannot verify signed message"
        );
        Ok(signed)
    }

    fn create_hash_bytes(&self, keys: &LocalAccount) -> anyhow::Result<Vec<u8>> {
        let sig = self.sign_bcs_bytes(keys).expect("could not sign bytes");

        make_commit_hash(self.entry_fee, self.epoch, sig.to_bytes().to_vec())
    }

    fn encode_commit_tx_payload(&self, keys: &LocalAccount) -> anyhow::Result<TransactionPayload> {
        let digest = self.create_hash_bytes(keys)?;
        let payload = libra_stdlib::secret_bid_commit(digest);
        Ok(payload)
    }

    // In the reveal we simply show the signature and
    // open bid info. On the MoveVM side the hash is recreated
    // so that it should match the commit
    fn encode_reveal_tx_payload(&self, keys: &LocalAccount) -> anyhow::Result<TransactionPayload> {
        let sig = self.sign_bcs_bytes(keys)?;
        let payload = libra_stdlib::secret_bid_reveal(
            keys.public_key().to_bytes().to_vec(),
            self.entry_fee,
            sig.to_bytes().to_vec(),
        );
        Ok(payload)
    }
}

pub async fn commit_reveal_poll(
    sender: &mut LibraSender,
    entry_fee: u64,
    delay_secs: u64,
) -> anyhow::Result<()> {
    println!("commit reveal bid: {}", entry_fee);
    let mut bid = PofBidData::new(entry_fee);
    let client = sender.client().clone();

    loop {
        let secs = libra_query::chain_queries::secs_remaining_in_epoch(&client).await?;
        info!("seconds remaining in epoch {secs}");

        // check what epoch we are in
        let _ = bid.update_epoch(&client).await;
        let must_reveal = libra_query::chain_queries::within_commit_reveal_window(&client).await?;

        let la = &sender.local_account;
        if must_reveal {
            info!("must reveal bid");
            let payload = bid.encode_reveal_tx_payload(la)?;
            // don't abort if we are trying too eager!
            match sender.sign_submit_wait(payload).await {
                Ok(_) => info!("successfully revealed bid"),
                Err(e) => error!(
                    "could not reveal bid: message {}\ncontinuing...",
                    e.to_string()
                ),
            }
        } else {
            info!("sending sealed bid");
            let payload = bid.encode_commit_tx_payload(la)?;
            match sender.sign_submit_wait(payload).await {
                Ok(_) => info!("successfully committed bid"),
                Err(e) => error!(
                    "could not commit bid: message {}\ncontinuing...",
                    e.to_string()
                ),
            }
        };

        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }
}

/// produces a hash by combining the bid and the signed bid.
/// On the move side there is an equivalent function at: secret_bid.move
pub fn make_commit_hash(
    entry_fee: u64,
    epoch: u64,
    mut signed_message: Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    let bid = PofBidData { entry_fee, epoch };

    let mut bid_bytes = bcs::to_bytes(&bid)?;
    bid_bytes.append(&mut signed_message);
    let commitment = HashValue::sha3_256_of(&bid_bytes);

    Ok(commitment.to_vec())
}

#[cfg(test)]
fn test_local_account() -> LocalAccount {
    use diem_sdk::crypto::ed25519::Ed25519PrivateKey;
    use diem_sdk::crypto::ValidCryptoMaterialStringExt;
    use diem_sdk::types::AccountKey;
    use libra_types::exports::AccountAddress;

    let pk = Ed25519PrivateKey::from_encoded_string(
        "74f18da2b80b1820b58116197b1c41f8a36e1b37a15c7fb434bb42dd7bdaa66b",
    )
    .unwrap();
    let account_key = AccountKey::from_private_key(pk);
    LocalAccount::new(
        AccountAddress::from_bytes(account_key.public_key().to_bytes()).unwrap(),
        account_key,
        0,
    )
}

#[test]
fn sign_bid() {
    let bid = PofBidData::new(11);
    let la = test_local_account();
    let sig = bid.sign_bcs_bytes(&la).expect("could not sign");
    let pubkey = la.public_key();
    let msg = bid.to_bcs().unwrap();

    assert!(sig.verify_arbitrary_msg(&msg, pubkey).is_ok());
}
