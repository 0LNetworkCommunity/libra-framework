use crate::stream::{channel, bid_commit_reveal::PofBidArgs, epoch_tickle_poll::epoch_tickle_poll};
use crate::submit_transaction::Sender as LibraSender;
use std::sync::{Arc, Mutex};

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
    pub fn start(&self, send: Arc<Mutex<LibraSender>>) {
        let (tx, rx) = channel::init_channel();
        let stream_service = channel::listen(rx, send.clone());

        match &self {
            StreamTxs::EpochTickle { delay } => {
                println!("EpochTickle entry");
                epoch_tickle_poll(tx, send, delay.unwrap_or(60));
            }
            StreamTxs::ValBid(args) => {
                let la = &send.lock().unwrap().local_account;

                let pubk = hex::encode(la.private_key().to_bytes());
                dbg!(&pubk);
                dbg!(&args);
            }
        };

        stream_service
            .join()
            .expect("could not complete tasks in stream");
    }
}
