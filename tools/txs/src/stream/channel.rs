
use crate::submit_transaction::Sender as LibraSender;
use diem_logger::prelude::{error, info};
use diem_types::transaction::TransactionPayload;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Creates a channel to receive an process for TransactionPayload
pub(crate) fn init_channel() -> (Sender<TransactionPayload>, Receiver<TransactionPayload>) {
    mpsc::channel::<TransactionPayload>()
}

/// Start the listener on the channel which receives TransactionPayload
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
                    info!("Received Channel Message\nTx: {:?}", payload);
                    if !busy {
                        busy = true;
                        match send
                            .lock()
                            .expect("could not access Sender client")
                            .sync_sign_submit_wait(payload)
                        {
                            Ok(r) => {
                                info!("tx success: {:?}", r);
                                busy = false;
                            }
                            Err(e) => {
                                // DOES NOT abort on failed tx
                                error!("transaction failed: {:?}\ncontinuing...", &e);
                            }
                        };
                    }
                }
                Err(_) => {
                  error!("could not parse channel message received, aborting");
                  break
                },
            };
        }
    })
}
