use crate::stream::bid_commit_reveal::commit_reveal_poll;
use crate::stream::{bid_commit_reveal::PofBidArgs, epoch_tickle_poll::epoch_tickle_poll};
use crate::submit_transaction::Sender as LibraSender;

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
    pub async fn start(&self, libra_sender: &mut LibraSender) -> anyhow::Result<()> {
        match &self {
            StreamTxs::EpochTickle { delay } => {
                println!("EpochTickle entry");
                epoch_tickle_poll(libra_sender, delay.unwrap_or(60)).await?;
            }
            StreamTxs::ValBid(args) => {
                commit_reveal_poll(libra_sender, args.net_reward, args.delay.unwrap_or(60)).await?;
            }
        }
        Ok(())
    }
}
