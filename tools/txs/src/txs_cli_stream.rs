use crate::stream::bid_commit_reveal::commit_reveal_poll;
use crate::stream::{bid_commit_reveal::PofBidArgs, channel, epoch_tickle_poll::epoch_tickle_poll};
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
    pub async fn start(&self, libra_sender: Arc<Mutex<LibraSender>>) {
        let (send_chan, receive_chan) = channel::init_channel();
        let stream_service = channel::listen(receive_chan, libra_sender.clone());

        match &self {
            StreamTxs::EpochTickle { delay } => {
                println!("EpochTickle entry");
                epoch_tickle_poll(send_chan, libra_sender, delay.unwrap_or(60))
                    .await
                    .expect("could not poll epoch boundary");
            }
            StreamTxs::ValBid(args) => commit_reveal_poll(
                send_chan,
                libra_sender,
                args.net_reward,
                args.delay.unwrap_or(60),
            )
            .await
            .expect("commit reveal poll ended"),
        };

        stream_service
            .join()
            .expect("could not complete tasks in stream");
    }
}
