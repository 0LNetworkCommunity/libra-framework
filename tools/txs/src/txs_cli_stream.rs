// use crate::bid_commit_reveal::PofBidArgs;
use crate::stream::epoch_tickle_poll::epoch_tickle_poll;
use crate::submit_transaction::Sender as LibraSender;
use diem_logger::prelude::{error, info};
use diem_types::transaction::TransactionPayload;
use std::process::exit;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[derive(clap::Subcommand)]
pub enum StreamTxs {
    /// Trigger the epoch boundary when available
    EpochTickle {
    /// optional, seconds delay between attempts, defaults to 60s
    #[clap(short, long)]
      delay: Option<u64>
    },
    /// Submit secret PoF bids in background, and reveal when window opens
    PofBid,
}

impl StreamTxs {
    pub fn start(&self, send: Arc<Mutex<LibraSender>>) {
        let (tx, rx) = init_channel();
        let client = send.lock().unwrap().client().clone();
        let stream_service = listen(rx, send);

        match &self {
            StreamTxs::EpochTickle { delay } => {
                println!("EpochTickle entry");
                epoch_tickle_poll(tx, client, delay.unwrap_or(60));
            }
            _ => {
                println!("no service specified");
                exit(1);
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
                            Ok(r) => {
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
