use crate::submit_transaction::Sender as LibraSender;
use diem_logger::{debug, error};
use diem_sdk::crypto::ed25519::Ed25519Signature;
use diem_sdk::crypto::Signature;
use diem_sdk::crypto::SigningKey;
use diem_sdk::types::LocalAccount;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use libra_query::chain_queries;
use libra_types::exports::Client;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
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

    fn encode_commit_tx_payload(&self, keys: &LocalAccount) -> TransactionPayload {
        let digest = self.sign_bcs_bytes(keys).expect("could not sign bytes");
        libra_stdlib::secret_bid_commit(digest.to_bytes().to_vec())
    }

    fn encode_reveal_tx_payload(&self, keys: &LocalAccount) -> TransactionPayload {
        let digest = self.sign_bcs_bytes(keys).unwrap();

        libra_stdlib::secret_bid_reveal(
            keys.public_key().to_bytes().to_vec(),
            self.entry_fee,
            digest.to_bytes().to_vec(),
        )
    }
}

pub async fn commit_reveal_poll(
    mut tx: Sender<TransactionPayload>,
    sender: Arc<Mutex<LibraSender>>,
    entry_fee: u64,
    delay_secs: u64,
) -> anyhow::Result<()> {
    println!("commit reveal bid: {}", entry_fee);
    let mut bid = PofBidData::new(entry_fee);

    loop {
        let client = sender.lock().unwrap().client().clone(); // releases the mutex

        // check what epoch we are in
        let _ = bid.update_epoch(&client).await;
        dbg!(&bid);
        debug!("bid: {:?}", &bid);
        let must_reveal = libra_query::chain_queries::within_commit_reveal_window(&client).await?;

        let la = &sender.lock().unwrap().local_account;
        let tx_payload = if must_reveal {
            bid.encode_reveal_tx_payload(la)
        } else {
            bid.encode_commit_tx_payload(la)
        };

        // send to channel
        match tx.borrow_mut().send(tx_payload.clone()) {
            Ok(_) => debug!("success with payload: {:?}", tx_payload),
            // Don't abort on error
            Err(e) => error!("transaction fails with message: {:?}\ncontinuing...", e),
        };
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }
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
