use crate::submit_transaction::Sender as LibraSender;
use diem_logger::info;
use diem_sdk::crypto::SigningKey;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use libra_query::chain_queries;
use libra_types::core_types::app_cfg::AppCfg;
use libra_types::exports::{Ed25519PrivateKey, Ed25519PublicKey};
use serde::{Deserialize, Serialize};
use libra_types::exports::Client;
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;

#[derive(clap::Args, Debug)]
pub struct PofBidArgs {
    #[clap(short, long)]
    pub net_reward: u64,

    #[clap(short, long)]
    pub test_private_key: Option<String>,
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
      epoch: 0
    }
  }
  fn to_bcs(&self) -> anyhow::Result<Vec<u8>>{
    Ok(bcs::to_bytes(&self)?)
  }

  fn from_bcs(bytes: &[u8]) -> anyhow::Result<Self> {
    Ok(bcs::from_bytes(bytes)?)
  }

  async fn update_epoch(&mut self, client: &Client) -> anyhow::Result<()>{
    let num = chain_queries::get_epoch(client).await?;
    self.epoch = num;
    Ok(())
  }

  fn sign_bcs_bytes(&self, private_key: Ed25519PrivateKey) -> anyhow::Result<Vec<u8>> {
    let bcs = self.to_bcs()?;
    let signed = private_key.sign_arbitrary_message(&bcs);
    Ok(signed.to_bytes().to_vec())
  }

}

pub fn commit_reveal_poll(mut tx: Sender<TransactionPayload>, sender: Arc<Mutex<LibraSender>>, delay_secs: u64, app_cfg: AppCfg) {
    println!("commit reveal bidding");
    let handle = thread::spawn(move || loop {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = sender.lock().unwrap().client().clone();
        // TODO: make the client borrow instead of clone
        let res = rt.block_on(libra_query::chain_queries::within_commit_reveal_window(
            &client,
        ));

        match res {
            Ok(true) => {
                let func = libra_stdlib::diem_governance_trigger_epoch();

                tx.borrow_mut().send(func).unwrap();
            }
            _ => {

                info!("Committing bid");

            }
        }

        thread::sleep(Duration::from_secs(delay_secs));
    });
    handle.join().expect("cannot poll for epoch boundary");
}


#[test]
fn encode_signed_message() {
  let pk = PrivateKey::from_bytes(hex::decode("74f18da2b80b1820b58116197b1c41f8a36e1b37a15c7fb434bb42dd7bdaa66b"));
  dbg!(&pk);
}
