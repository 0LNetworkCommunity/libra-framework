
use crate::stream::{
  bid_commit_reveal::PofBidArgs,
  epoch_tickle_poll::epoch_tickle_poll
};
use crate::submit_transaction::Sender as LibraSender;
use diem_logger::prelude::{error, info};
use diem_types::transaction::TransactionPayload;
use libra_types::core_types::app_cfg::AppCfg;
use libra_types::exports::Ed25519PrivateKey;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[derive(clap::Subcommand)]
pub enum StreamTxs {
    /// Trigger the epoch boundary when available
    EpochTickle {
        /// optional, seconds delay between attempts, defaults to 60s
        #[clap(short, long)]
        delay: Option<u64>,
    },
    /// Submit secret PoF bids in background, and reveal when window opens
    ValBid(PofBidArgs),
}

impl StreamTxs {
    pub fn start(&self, send: Arc<Mutex<LibraSender>>, _app_cfg: &AppCfg) {
        let (tx, rx) = init_channel();
        let stream_service = listen(rx, send.clone());

        match &self {
            StreamTxs::EpochTickle { delay } => {
                println!("EpochTickle entry");
                epoch_tickle_poll(tx, send, delay.unwrap_or(60));
            }
            StreamTxs::ValBid(args) => {

              dbg!(&args);
            }
        };

        stream_service
            .join()
            .expect("could not complete tasks in stream");
    }
}

pub(crate) fn init_channel() -> (Sender<TransactionPayload>, Receiver<TransactionPayload>) {
    mpsc::channel::<TransactionPayload>()
}

#[allow(unused_assignments)]
pub(crate) fn listen(
    rx: Receiver<TransactionPayload>,
    send: Arc<Mutex<LibraSender>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut busy = false;

        while !busy {
            match rx.recv() {
                Ok(payload) => {
                    info!("Tx: {:?}", payload);
                    if !busy {
                        busy = true;
                        match send
                            .lock()
                            .expect("could not access Sender client")
                            .sync_sign_submit_wait(payload)
                        {
                            Ok(_r) => {
                                busy = false;
                            }
                            Err(e) => {
                                error!("transaction failed: {:?}", &e);
                                break;
                            }
                        };
                    }
                }
                Err(_) => break,
            };
        }
    })
}
