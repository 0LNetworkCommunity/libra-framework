//! Validator subcommands

use crate::submit_transaction::Sender;
use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;

#[derive(clap::Subcommand)]
pub enum CommunityTxs {
    /// Propose a Tx
    Propose(ProposeTx),
    /// Donors to Donor Voice addresses (like Community Wallets), can vote to
    /// reject transactions.
    Veto(VetoTx),
    /// initialize a DonorVoice multisig with the initial admins.
    GovInit(InitTx),
    /// propose a change to the authorities of the DonorVoice multisig
    GovAdmin(AdminTx),
}

impl CommunityTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        match &self {
            CommunityTxs::Propose(propose) => match propose.run(sender).await {
                Ok(_) => println!("SUCCESS: community wallet transfer proposed"),
                Err(e) => {
                    println!("ERROR: community wallet transfer rejected, message: {}", e);
                }
            },
            CommunityTxs::Veto(veto) => match veto.run(sender).await {
                Ok(_) => println!("SUCCESS: veto vote submitted"),
                Err(e) => {
                    println!("ERROR: veto vote rejected, message: {}", e);
                }
            },
            CommunityTxs::GovInit(init) => match init.run(sender).await {
                Ok(_) => println!("SUCCESS: community wallet initialized"),
                Err(e) => {
                    println!(
                        "ERROR: could not initialize Community Wallet, message: {}",
                        e
                    );
                }
            },
            CommunityTxs::GovAdmin(admin) => match admin.run(sender).await {
                Ok(_) => println!("SUCCESS: community wallet admin added"),
                Err(e) => {
                    println!("ERROR: could not add admin, message: {}", e);
                }
            },
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct ProposeTx {
    #[clap(short, long)]
    /// The Community Wallet you are a admin for
    community_wallet: AccountAddress,
    #[clap(short, long)]
    /// The SlowWallet recipient of funds
    recipient: AccountAddress,
    #[clap(short, long)]
    /// amount of coins (units) to transfer
    amount: u64,
    #[clap(short, long)]
    /// description of payment for memo
    description: String,
}

impl ProposeTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::donor_voice_propose_payment_tx(
            self.community_wallet,
            self.recipient,
            self.amount,
            self.description.clone().into_bytes(),
        );
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct VetoTx {
    #[clap(short, long)]
    /// The SlowWallet recipient of funds
    community_wallet: AccountAddress,
    #[clap(short, long)]
    /// Proposal number
    proposal_id: u64,
}

impl VetoTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload =
            libra_stdlib::donor_voice_propose_veto_tx(self.community_wallet, self.proposal_id);
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct InitTx {
    #[clap(short, long)]
    /// The initial admins of the Multisig. Note: the signer of this TX
    /// (sponsor) cannot add self.
    admins: Vec<AccountAddress>,
}

impl InitTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::community_wallet_init_community(self.admins.clone());
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct AdminTx {
    #[clap(short, long)]
    /// The SlowWallet recipient of funds
    community_wallet: AccountAddress,
    #[clap(short, long)]
    /// Admin to add (or remove) from the multisig
    admin: AccountAddress,
    #[clap(short, long)]
    /// Drops this admin from the multisig
    drop: bool,
    #[clap(short, long)]
    /// Number of sigs required for action (must be greater than 3-of-5)
    n: u64,
    #[clap(short, long)]
    /// Proposal duration (in epochs)
    epochs: Option<u64>,
}

impl AdminTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::community_wallet_add_signer_community_multisig(
            self.community_wallet,
            self.admin,
            self.n,
            self.epochs.unwrap_or(10), // todo: remo
        );
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}
