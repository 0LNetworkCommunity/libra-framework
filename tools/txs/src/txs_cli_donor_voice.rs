//! Validator subcommands

use crate::submit_transaction::Sender;
use diem_types::{
    account_address::AccountAddress
};
use libra_cached_packages::libra_stdlib;
use libra_types::{
    exports::{AuthenticationKey, Ed25519PrivateKey},
    type_extensions::client_ext::ClientExt,
};
use libra_wallet::account_keys::get_keys_from_prompt;

#[derive(clap::Subcommand)]
pub enum DonorVoiceTxs {
    /// Propose a Tx
    Propose(ProposeTx),
    /// initialize a DonorVoice multisig with the initial admins.
    GovInit(InitTx),
    /// propose a change to the authorities of the DonorVoice multisig
    GovAdmins(AdminsTx),
}

impl DonorVoiceTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        match &self {
            DonorVoiceTxs::Propose(rotate) => match rotate.run(sender).await {
                Ok(_) => println!("SUCCESS: private key rotated"),
                Err(e) => {
                    println!("ERROR: could not rotate private key, message: {}", e);
                }
            },
            DonorVoiceTxs::GovInit(slow) => match slow.run(sender).await {
                Ok(_) => println!("SUCCESS: account set to Slow Wallet"),
                Err(e) => {
                    println!(
                        "ERROR: could set the account to Slow Wallet, message: {}",
                        e
                    );
                }
            },
            DonorVoiceTxs::GovAdmins(slow) => match slow.run(sender).await {
                Ok(_) => println!("SUCCESS: account set to Slow Wallet"),
                Err(e) => {
                    println!(
                        "ERROR: could set the account to Slow Wallet, message: {}",
                        e
                    );
                }
            },
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct ProposeTx {
    #[clap(short, long)]
    /// The SlowWallet recipient of funds
    recipient: AccountAddress,
    /// amount of coins (units) to transfer
    amount: u64,
}

impl ProposeTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::slow_wallet_user_set_slow();
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct InitTx {
    #[clap(short, long)]
    /// Set this with governance defaults of a "Community Wallet"
    community: bool, // Dev NOTE: account address has the
    #[clap(short, long)]
    /// The initial admins of the Multisig
    init_admins: Vector<AccountAddress>, // Dev NOTE: account address has the same bytes as AuthKey
}

impl InitTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::slow_wallet_user_set_slow();
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct AdminsTx {
    #[clap(short, long)]
    /// Uses Community Wallet multisig governance
    community: bool,
    #[clap(short, long)]
    /// The initial admins of the Multisig
    init_admins: Vector<AccountAddress>, // Dev NOTE: account address has the same bytes as AuthKey
}

impl AdminsTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        if self.community {
            let payload = libra_stdlib::slow_wallet_user_set_slow();
            sender.sign_submit_wait(payload).await?;
        } else {
            not_implemented!()
        }

        Ok(())
    }
}
