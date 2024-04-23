//! Validator subcommands
use crate::submit_transaction::Sender;

use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;
use libra_query::query_view;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(clap::Subcommand)]
pub enum CommunityTxs {
    /// Propose a multi-sig transaction
    Propose(ProposeTx),
    /// Execute batch proposals/approvals of transactions
    Batch(BatchTx),
    /// Donors to Donor Voice addresses can vote to reject transactions
    Veto(VetoTx),
    /// Initialize a DonorVoice multi-sig. NOTE: this is a two step procedure:
    /// propose the admins, and then rotate the account keys with --finalize
    GovInit(InitTx),
    /// Propose a change to the authorities of the DonorVoice multi-sig
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
            CommunityTxs::Batch(batch) => match batch.run(sender).await {
                Ok(_) => {
                    println!("Batch transactions submitted, see log in batch_log.json, some TXS may have failed (not atomic)")
                }
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
    /// The Community Wallet to schedule transaction
    pub community_wallet: AccountAddress,
    #[clap(short, long)]
    /// The SlowWallet recipient of funds
    pub recipient: AccountAddress,
    #[clap(short, long)]
    /// Amount of coins (units) to transfer
    pub amount: u64,
    #[clap(short, long)]
    /// Description of payment for memo
    pub description: String,
}

impl ProposeTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::donor_voice_txs_propose_payment_tx(
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
pub struct BatchTx {
    #[clap(short, long)]
    /// The Community Wallet to schedule transaction
    pub community_wallet: AccountAddress,
    #[clap(short, long)]
    /// JSON file with batch payments
    pub file: PathBuf,
    #[clap(short, long)]
    /// Write the result json to a different file (otherwise will overwrite)
    pub out: Option<PathBuf>,
    #[clap(long)]
    /// Just check if the destinations are slow wallets
    pub check: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProposePay {
    recipient: AccountAddress,
    amount: u64,
    description: String,
    is_slow: Option<bool>,
    success: Option<bool>,
    error: Option<String>,
    note: Option<String>,
}

impl BatchTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let data = fs::read_to_string(&self.file).expect("Unable to read file");
        let mut list: Vec<ProposePay> = serde_json::from_str(&data).expect("Unable to parse");

        // TODO: use an iter_mut here. Async will be annoying.
        for inst in &mut list {
            println!("account: {:?}", &inst.recipient);

            if let Some(success) = inst.success {
                if success {
                    println!("...skipping already successful transaction");
                    continue;
                };
            }

            let res_slow = query_view::get_view(
                &sender.client(),
                "0x1::slow_wallet::is_slow",
                None,
                Some(inst.recipient.to_string()),
            )
            .await?;
            dbg!(&res_slow);
            if self.check {
                continue;
            };

            println!("scheduling tx");
            match propose_single(sender, &self.community_wallet, &inst).await {
                Ok(_) => {
                    inst.success = Some(true);
                }
                Err(e) => {
                    println!("transaction failed");
                    inst.success = Some(false);
                    inst.error = Some(e.to_string())
                }
            }
        }

        let json = serde_json::to_string(&list)?;
        let p = if let Some(out_path) = &self.out {
            out_path
        } else {
            println!("overwriting {}", &self.file.display());
            &self.file
        };

        fs::write(p, json)?;

        Ok(())
    }
}

async fn propose_single(
    sender: &mut Sender,
    multisig: &AccountAddress,
    instruction: &ProposePay,
) -> anyhow::Result<()> {
    let payload = libra_stdlib::donor_voice_txs_propose_payment_tx(
        multisig.to_owned(),
        instruction.recipient,
        instruction.amount,
        instruction.description.clone().into_bytes(),
    );
    sender.sign_submit_wait(payload).await?;
    Ok(())
}

#[derive(clap::Args)]
pub struct VetoTx {
    #[clap(short, long)]
    /// The Slow Wallet recipient of funds
    pub community_wallet: AccountAddress,
    #[clap(short, long)]
    /// Proposal number
    pub proposal_id: u64,
}

impl VetoTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload =
            libra_stdlib::donor_voice_txs_propose_veto_tx(self.community_wallet, self.proposal_id);
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}

#[derive(clap::Args)]
/// Initialize a community wallet in two steps 1) make it a donor voice account,
/// and check proposed authorities 2) finalize and set the authorities
pub struct InitTx {
    #[clap(short, long)]
    /// The initial admins of the multi-sig (cannot add self)
    pub admins: Vec<AccountAddress>,

    #[clap(short, long)]
    /// Num of signatures needed for the n-of-m
    pub num_signers: u64,

    #[clap(long)]
    /// Finalize the configurations and rotate the auth key, not reversible!
    pub finalize: bool,
}

impl InitTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        if self.finalize {
            // Warning message
            println!("\nWARNING: This operation will finalize the account associated with the governance-initialized wallet and make it inaccessible. This action is IRREVERSIBLE and can only be applied to a wallet where governance has been initialized.\n");

            // Assuming the signer's account is already set in the `sender` object
            // The payload for the finalize and cage operation
            let payload =
                libra_stdlib::multi_action_finalize_and_cage(self.admins.clone(), self.num_signers); // This function now does not require an account address

            // Execute the transaction
            sender.sign_submit_wait(payload).await?;
            println!("The account has been finalized and caged.");
        } else {
            let payload = libra_stdlib::community_wallet_init_init_community(
                self.admins.clone(),
                self.num_signers,
            );

            sender.sign_submit_wait(payload).await?;
            println!("You have completed the first step in creating a community wallet, now you should check your work and finalize with --finalize");
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct AdminTx {
    #[clap(short, long)]
    /// The SlowWallet recipient of funds
    pub community_wallet: AccountAddress,
    #[clap(short, long)]
    /// Admin to add (or remove) from the multisig
    pub admin: AccountAddress,
    #[clap(short, long)]
    /// Drops this admin from the multisig
    pub drop: Option<bool>,
    #[clap(short, long)]
    /// Number of sigs required for action (must be greater than 3-of-5)
    pub n: u64,
    #[clap(short, long)]
    /// Proposal duration (in epochs)
    pub epochs: Option<u64>,
}

impl AdminTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        // Default to adding a signer if the `drop` flag is not provided
        let is_add_operation = self.drop.unwrap_or(true);

        let payload = libra_stdlib::community_wallet_init_change_signer_community_multisig(
            self.community_wallet,
            self.admin,
            is_add_operation,
            self.n,
            self.epochs.unwrap_or(10), // todo: remo
        );
        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}
