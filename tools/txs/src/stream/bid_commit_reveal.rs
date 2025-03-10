use crate::submit_transaction::Sender as LibraSender;
use diem_logger::info;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use libra_query::chain_queries;
use libra_types::core_types::app_cfg::AppCfg;
use serde::{Deserialize, Serialize};
use libra_types::exports::Client;
use std::borrow::BorrowMut;
use std::sync::mpsc::Sender;
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

  // fn sign_bcs_bytes(&self, public_key) -> anyhow::Result<Vec<u8>> {
  //   let bcs = self.to_bcs();

  //   Ok(bcs)
  // }

}

pub fn commit_reveal_poll(mut tx: Sender<TransactionPayload>, client: Client, delay_secs: u64, app_cfg: AppCfg) {
    println!("polling epoch boundary");
    let handle = thread::spawn(move || loop {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // TODO: make the client borrow instead of clone
        let res = rt.block_on(libra_query::chain_queries::within_commit_reveal_window(
            &client.clone(),
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
