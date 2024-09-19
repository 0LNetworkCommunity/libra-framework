// use crate::bid_commit_reveal::PofBidArgs;
use crate::submit_transaction::Sender as LibraSender;
use diem_types::transaction::TransactionPayload;
use libra_cached_packages::libra_stdlib;
use diem_logger::prelude::{info, error};
use std::time::Duration;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

#[derive(clap::Subcommand)]
pub enum StreamTxs {
    // PofBid(PofBidArgs),
    /// Submit secret PoF bids in background, and reveal when window opens
    PofBid
}

impl StreamTxs {
  pub fn start(&self, send: Arc<Mutex<LibraSender>>) {
    let (mut tx, rx) = init_channel();
    let stream_service = listen(rx, send);

    let _worker = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1000));
        //         if libra_query::chain_queries::is_gov_proposal_resolved(
        //     sender.client(),
        //     *proposal_id,
        // )
        let func = libra_stdlib::diem_governance_trigger_epoch();

        tx.borrow_mut().send(func).unwrap();
    });

    stream_service
        .join()
        .expect("could not complete tasks in stream");
  }

}

pub(crate) fn init_channel() -> (Sender<TransactionPayload>, Receiver<TransactionPayload>) {
    mpsc::channel::<TransactionPayload>()
}

// pub fn demo_send(tx: &Sender<TradeTask>) {
//     // Push data to the channel
//     let task1 = TradeTask::new(1.0, 2.0, "first task".to_string());
//     let task2 = TradeTask::new(3.1, 4.1, "second task".to_string());
//     tx.send(task1).unwrap();
//     tx.send(task2).unwrap();
// }

pub(crate) fn listen(rx: Receiver<TransactionPayload>, send: Arc<Mutex<LibraSender>>) -> JoinHandle<()> {
    let worker = thread::spawn(move || {
      let mut busy = false;

      while !busy {
        match rx.recv() {
            Ok(payload) => {
              info!("Tx: {:?}", payload);
              if !busy {
                busy = true;
                let _ = match send.lock().expect("could not access Sender client").sync_sign_submit_wait(payload) {
                  Ok(r) => {
                    dbg!(&r);
                    busy = false;
                  },
                  Err(e) => {
                    error!("transaction failed: {:?}", &e);
                    break
                  },
                };
              }

            },
            Err(_) => break,
        };
      }
    });
    worker
}
