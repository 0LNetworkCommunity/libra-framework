#[derive(clap::Args)]
pub struct PofBidArgs {
    #[clap(short, long)]
    pub net_reward: u64,

    #[clap(short, long)]
    pub test_private_key: Option<String>,
}
