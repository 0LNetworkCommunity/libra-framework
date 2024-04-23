//! Validator subcommands
use crate::submit_transaction::Sender;

use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;
use libra_query::{account_queries, query_view};
use libra_types::move_resource::gas_coin;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

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
            gas_coin::cast_decimal_to_coin(self.amount as f64),
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
    recipient: String,
    parsed: Option<AccountAddress>,
    amount: u64,
    description: String,
    is_slow: Option<bool>,
    proposed: Option<bool>,
    voters: Option<Vec<AccountAddress>>,
    error: Option<String>,
    note: Option<String>,
}

// DEV NOTE: really what we should be doing is creating a Move transaction
// script that submits all TXS in a batch and executes all or aborts
// (an atomic batch).
impl BatchTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let data = fs::read_to_string(&self.file).expect("Unable to read file");
        let mut list: Vec<ProposePay> = serde_json::from_str(&data).expect("Unable to parse");

        let ballots =
            account_queries::multi_auth_ballots(sender.client(), self.community_wallet).await?;
        let d = ballots.as_object().unwrap();
        let v = d.get("vote").unwrap().as_object().unwrap();
        let p = v.get("ballots_pending").unwrap().as_array().unwrap();

        let mut pending: HashMap<AccountAddress, ProposePay> = HashMap::new();
        p.iter().for_each(|e| {
            let o = e.as_object().unwrap();
            let prop = o.get("tally_type").unwrap().as_object().unwrap();
            let data = prop.get("proposal_data").unwrap().as_object().unwrap();
            let recipient: AccountAddress = data
                .get("payee")
                .unwrap()
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let amount: u64 = data
                .get("value")
                .unwrap()
                .as_str()
                .unwrap()
                .parse()
                .unwrap();

            let voters: Vec<AccountAddress> = prop
                .get("votes")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e.as_str().unwrap().parse::<AccountAddress>().unwrap())
                .collect();

            let found = ProposePay {
                recipient: recipient.to_canonical_string(),
                parsed: Some(recipient),
                amount,
                description: "debugging".to_string(),
                is_slow: None,
                proposed: None,
                voters: Some(voters),
                error: None,
                note: None,
            };

            pending.insert(recipient, found);
        });

        for inst in &mut list {
            let addr: AccountAddress = inst.recipient.parse().expect(&format!("could not parse {}", &inst.recipient));

            inst.parsed = Some(addr.clone());

            println!("account: {:?}", &inst.recipient);

            if let Some((_, pp)) = pending.get_key_value(&addr) {
                if pp.amount == inst.amount {
                    inst.proposed = Some(true);
                    inst.voters = pp.voters.clone();
                    println!("... found already pending, mark as proposed");
                }
            };

            if let Some(v) = &inst.voters {
                if v.contains(&sender.local_account.address()) {
                    println!("... already voted, skipping");
                    continue;
                }
            }

            let res_slow = query_view::get_view(
                &sender.client(),
                "0x1::slow_wallet::is_slow",
                None,
                Some(inst.recipient.to_string()),
            )
            .await?
            .as_array()
            .unwrap()[0]
                .as_bool()
                .unwrap();

            inst.is_slow = Some(res_slow);
            if !res_slow {
                println!("... is not a slow wallet, skipping");
            }

            if self.check {
                continue;
            };

            println!("scheduling tx");

            match propose_single(sender, &self.community_wallet, &inst).await {
                Ok(_) => {
                    inst.proposed = Some(true);
                }
                Err(e) => {
                    println!("transaction failed");
                    inst.proposed = Some(false);
                    inst.error = Some(e.to_string())
                }
            }
        }

        if self.check {
            list.iter().for_each(|e| {
                if let Some(s) = e.is_slow {
                    if !s {
                        println!("not slow: {} : {}", e.note.as_ref().unwrap_or(&"n/a".to_string()), e.recipient.to_string());
                    }
                }
            });
        };

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
        instruction.parsed.unwrap(),
        gas_coin::cast_decimal_to_coin(instruction.amount as f64),
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
