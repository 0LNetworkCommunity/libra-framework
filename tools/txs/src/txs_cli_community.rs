//! Validator subcommands

use crate::submit_transaction::Sender;
use diem_logger::error;
use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;
use libra_query::{account_queries, query_view};
use libra_types::move_resource::gas_coin;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(clap::Subcommand)]
pub enum CommunityTxs {
    /// Initialize a DonorVoice multi-sig by proposing an offer to initial authorities.
    //  NOTE: Then authorities need to claim the offer, and the donor have to cage the account to become a multi-sig account.
    GovInit(InitTx),
    /// Update proposed offer to initial authorities
    GovOffer(OfferTx),
    /// Claim the proposed offer
    GovClaim(ClaimTx),
    /// Finalize and cage the multisig account after authorities claim the offer
    GovCage(CageTx),
    /// Propose a change to the authorities of the DonorVoice multi-sig
    GovAdmin(AdminTx),
    /// Propose a multi-sig transaction
    Propose(ProposeTx),
    /// Execute batch proposals/approvals of transactions
    Batch(BatchTx),
    /// Donors to Donor Voice addresses can vote to reject transactions
    Veto(VetoTx),
    /// Donor can vote in reauthorization poll
    Reauthorize(ReauthVoteTx),
}

impl CommunityTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let result = match self {
            CommunityTxs::GovInit(tx) => {
                tx.run(sender).await.map(|_| "community wallet initialized")
            }
            CommunityTxs::GovOffer(tx) => tx
                .run(sender)
                .await
                .map(|_| "community wallet offer proposed"),
            CommunityTxs::GovClaim(tx) => tx
                .run(sender)
                .await
                .map(|_| "community wallet offer claimed"),
            CommunityTxs::GovCage(tx) => tx.run(sender).await.map(|_| "community wallet finalized"),
            CommunityTxs::GovAdmin(tx) => tx
                .run(sender)
                .await
                .map(|_| "community wallet admin proposed"),
            CommunityTxs::Propose(tx) => tx
                .run(sender)
                .await
                .map(|_| "community wallet transfer proposed"),
            CommunityTxs::Veto(tx) => tx.run(sender).await.map(|_| "veto vote submitted"),
            CommunityTxs::Batch(tx) => tx
                .run(sender)
                .await
                .map(|_| "batch of transactions proposed"),
            CommunityTxs::Reauthorize(tx) => {
                tx.run(sender).await.map(|_| "reauthorize vote submitted")
            }
        };

        match result {
            Ok(message) if !message.is_empty() => println!("SUCCESS: {}", message),
            Err(e) => error!("Operation failed: {}", e),
            _ => {}
        }

        Ok(())
    }
}

#[derive(clap::Args)]
/// Initialize a community wallet offering the initial authorities
pub struct InitTx {
    #[clap(short, long)]
    /// The initial admins of the multi-sig (cannot add self)
    pub admins: Vec<AccountAddress>,

    #[clap(short, long)]
    /// Num of signatures needed for the n-of-m
    pub num_signers: u64,
}

impl InitTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::community_wallet_init_init_community(
            self.admins.clone(),
            self.num_signers,
        );

        sender.sign_submit_wait(payload).await?;
        println!("You have completed the first step in creating a community wallet, now the authorities you have proposed need to claim the offer.");

        Ok(())
    }
}

#[derive(clap::Args)]
/// Propose offer to authorities to become an authority in the community wallet
pub struct OfferTx {
    #[clap(short, long)]
    /// The Community Wallet to propose the offer
    pub admins: Vec<AccountAddress>,
    /// Num of signatures needed for the n-of-m
    pub num_signers: u64,
}

impl OfferTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::community_wallet_init_propose_offer(
            self.admins.clone(),
            self.num_signers,
        );
        sender.sign_submit_wait(payload).await?;
        println!("You have proposed the community wallet offer to the authorities.");
        Ok(())
    }
}

#[derive(clap::Args)]
/// Claim the offer to become an authority in the multi-sig
pub struct ClaimTx {
    #[clap(short, long)]
    /// The Community Wallet to claim the offer
    pub community_wallet: AccountAddress,
}

impl ClaimTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::multi_action_claim_offer(self.community_wallet);
        sender.sign_submit_wait(payload).await?;
        println!("You have claimed the community wallet offer.");
        Ok(())
    }
}

#[derive(clap::Args)]
/// Finalize and cage the community wallet to become multisig
pub struct CageTx {
    #[clap(short, long)]
    /// Num of signatures needed for the n-of-m
    pub num_signers: u64,
}

impl CageTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = libra_stdlib::community_wallet_init_finalize_and_cage(self.num_signers);
        sender.sign_submit_wait(payload).await?;
        println!("The community wallet is finalized and caged. It is now a multi-sig account.");
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
    #[clap(short, long)]
    /// Request small administrative advance
    pub advance: bool,
}

impl ProposeTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = if self.advance {
            libra_stdlib::donor_voice_txs_propose_advance_tx(
                self.community_wallet,
                self.recipient,
                gas_coin::cast_decimal_to_coin(self.amount as f64),
                self.description.clone().into_bytes(),
            )
        } else {
            libra_stdlib::donor_voice_txs_propose_payment_tx(
                self.community_wallet,
                self.recipient,
                gas_coin::cast_decimal_to_coin(self.amount as f64),
                self.description.clone().into_bytes(),
            )
        };
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

/// Used for batch processing of CW payments
#[derive(Serialize, Deserialize, Clone)]
struct ProposePay {
    recipient: String,
    parsed: Option<AccountAddress>,
    amount: u64,
    description: String,
    is_advance: bool,
    is_slow: Option<bool>,
    proposed: Option<bool>,
    approved: Option<bool>,
    voters: Option<Vec<AccountAddress>>,
    error: Option<String>,
    note: Option<String>,
}

// DEV NOTE: really what we should be doing is creating a Move transaction
// script that submits all TXS in a batch and executes all or aborts
// (an atomic batch).
impl BatchTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let data = fs::read_to_string(&self.file)?;
        let mut list: Vec<ProposePay> = serde_json::from_str(&data)?;

        let ballots =
            account_queries::multi_auth_ballots(sender.client(), self.community_wallet).await?;

        let mut pending_or_approved = HashMap::new();
        if let Some(d) = ballots.as_object() {
            if let Some(v) = d.get("vote").and_then(|v| v.as_object()) {
                let mut approved = v
                    .get("ballots_approved")
                    .and_then(|a| a.as_array())
                    .cloned()
                    .unwrap_or_default();

                let mut pending = v
                    .get("ballots_pending")
                    .and_then(|p| p.as_array())
                    .cloned()
                    .unwrap_or_default();

                pending.append(&mut approved);

                for ballot in pending {
                    if let Some(ballot_obj) = ballot.as_object() {
                        if let Some(prop) = ballot_obj.get("tally_type").and_then(|t| t.as_object())
                        {
                            if let Some(data) =
                                prop.get("proposal_data").and_then(|p| p.as_object())
                            {
                                if let (Some(recipient_str), Some(amount_str)) = (
                                    data.get("payee").and_then(|p| p.as_str()),
                                    data.get("value").and_then(|v| v.as_str()),
                                ) {
                                    if let (Ok(recipient), Ok(amount)) = (
                                        recipient_str.parse::<AccountAddress>(),
                                        amount_str.parse::<u64>(),
                                    ) {
                                        let voters: Vec<AccountAddress> = prop
                                            .get("votes")
                                            .and_then(|v| v.as_array())
                                            .map(|votes| {
                                                votes
                                                    .iter()
                                                    .filter_map(|e| e.as_str())
                                                    .filter_map(|s| s.parse().ok())
                                                    .collect()
                                            })
                                            .unwrap_or_default();

                                        let is_approved = prop
                                            .get("approved")
                                            .and_then(|a| a.as_bool())
                                            .unwrap_or(false);

                                        pending_or_approved.insert(
                                            recipient,
                                            ProposePay {
                                                recipient: recipient.to_canonical_string(),
                                                parsed: Some(recipient),
                                                amount,
                                                description: "debugging".to_string(),
                                                is_advance: false, // TODO: should warn that advance txs are not available in batch.

                                                is_slow: None,
                                                proposed: None,
                                                approved: Some(is_approved),
                                                voters: Some(voters),
                                                error: None,
                                                note: None,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for inst in &mut list {
            let addr = match inst.recipient.parse::<AccountAddress>() {
                Ok(addr) => addr,
                Err(_) => {
                    error!("Could not parse address: {}", inst.recipient);
                    continue;
                }
            };

            inst.parsed = Some(addr);
            println!("account: {:?}", &inst.recipient);

            // Check if this instruction already exists
            if let Some(pp) = pending_or_approved.get(&addr) {
                if pp.amount == gas_coin::cast_decimal_to_coin(inst.amount as f64) {
                    inst.proposed = Some(true);
                    inst.voters.clone_from(&pp.voters);
                    inst.approved = pp.approved;
                    println!("... found already pending, mark as proposed");
                }
            };

            // Check if it's a slow wallet
            let res_slow = query_view::get_view(
                sender.client(),
                "0x1::slow_wallet::is_slow",
                None,
                Some(inst.recipient.clone()),
            )
            .await?
            .as_array()
            .and_then(|arr| arr.first()?.as_bool())
            .unwrap_or(false);

            inst.is_slow = Some(res_slow);
            if !res_slow {
                println!("... is not a slow wallet, skipping");
                continue;
            }

            // Skip if already voted
            if let Some(voters) = &inst.voters {
                if voters.contains(&sender.local_account.address()) {
                    println!("... already voted, skipping");
                    continue;
                }
            }

            if self.check {
                continue;
            };

            println!("scheduling tx");

            match propose_single(sender, &self.community_wallet, inst).await {
                Ok(_) => {
                    inst.proposed = Some(true);
                }
                Err(e) => {
                    error!("Transaction failed: {}", e);
                    inst.proposed = Some(false);
                    inst.error = Some(e.to_string());
                }
            }
        }

        if self.check {
            for item in &list {
                if let Some(is_slow) = item.is_slow {
                    if !is_slow {
                        println!(
                            "not slow: {} : {}",
                            item.note.as_deref().unwrap_or("n/a"),
                            item.recipient
                        );
                    }
                }
            }
            println!("checks completed");
        } else {
            println!("Transfers proposed and voted on. Note: transactions are not atomic, some of the transfers may have been ignored. JSON file will be updated.");
        }

        let json = serde_json::to_string(&list)?;
        let output_path = self.out.as_ref().unwrap_or(&self.file);

        if self.out.is_none() {
            println!("overwriting {}", self.file.display());
        }

        fs::write(output_path, json)?;

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
pub struct ReauthVoteTx {
    #[clap(short, long)]
    /// The Slow Wallet recipient of funds
    pub community_wallet: AccountAddress,
}

impl ReauthVoteTx {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        // we'll try to tally the poll, this should never abort.
        let payload = libra_stdlib::donor_voice_txs_maybe_tally_reauth_tx(self.community_wallet);
        sender.sign_submit_wait(payload).await?;

        let payload = libra_stdlib::donor_voice_txs_vote_reauth_tx(self.community_wallet);
        sender.sign_submit_wait(payload).await.ok();

        // Call the view function to show current tally
        let res = query_view::get_view(
            sender.client(),
            "0x1::donor_voice_governance::get_reauth_tally",
            None,
            Some(self.community_wallet.to_canonical_string()),
        )
        .await?;

        display_reauth_tally_results(&res);

        Ok(())
    }
}

/// Displays the reauthorization poll results in a readable format
fn display_reauth_tally_results(res: &serde_json::Value) {
    // Parse the response according to the documented structure
    if let Some(arr) = res.as_array() {
        if arr.len() >= 7 {
            // Extract and format percentage values (divide by 100 for display)
            let percent_approval = arr[0]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0) as f64
                / 100.0;
            let turnout_percent = arr[1]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0) as f64
                / 100.0;
            let threshold_needed = arr[2]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0) as f64
                / 100.0;
            let epoch_deadline = arr[3]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let min_turnout_required = arr[4]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0) as f64
                / 100.0;
            let approved = arr[5].as_bool().unwrap_or(false);
            let is_complete = arr[6].as_bool().unwrap_or(false);

            println!("\nReauthorization Poll Status:");
            println!("------------------------------");
            println!("Approval Rate:       {:.2}%", percent_approval);
            println!("Voter Turnout:       {:.2}%", turnout_percent);
            println!("Approval Threshold:  {:.2}%", threshold_needed);
            println!("Minimum Turnout:     {:.2}%", min_turnout_required);
            println!("Epoch Deadline:      {}", epoch_deadline);
            println!(
                "Poll Complete:       {}",
                if is_complete { "Yes" } else { "No" }
            );

            if is_complete {
                println!(
                    "Result:              {}",
                    if approved { "APPROVED" } else { "REJECTED" }
                );

                // Add explanation for rejection if the poll was not approved
                if !approved {
                    let approval_passing = percent_approval >= threshold_needed;
                    let turnout_passing = turnout_percent >= min_turnout_required;

                    println!("Rejection Reason:    ");
                    if !approval_passing && !turnout_passing {
                        println!("                    • Both approval rate ({:.2}% < {:.2}%) and turnout ({:.2}% < {:.2}%) below thresholds",
                                 percent_approval, threshold_needed, turnout_percent, min_turnout_required);
                    } else if !approval_passing {
                        println!("                    • Approval rate too low: {:.2}% (threshold: {:.2}%)",
                                 percent_approval, threshold_needed);
                    } else if !turnout_passing {
                        println!(
                            "                    • Voter turnout too low: {:.2}% (minimum: {:.2}%)",
                            turnout_percent, min_turnout_required
                        );
                    } else {
                        println!(
                            "                    • Unknown reason (possible logic error in tally)"
                        );
                    }
                }
            } else {
                // Provide more detailed status about passing/failing conditions
                let approval_passing = percent_approval >= threshold_needed;
                let turnout_passing = turnout_percent >= min_turnout_required;

                if approval_passing && turnout_passing {
                    println!("Current Status:     On track to PASS");
                } else {
                    println!("Current Status:     Not passing requirements");

                    if !approval_passing && !turnout_passing {
                        println!("                    • Both approval rate ({:.2}% < {:.2}%) and turnout ({:.2}% < {:.2}%) below thresholds",
                                percent_approval, threshold_needed, turnout_percent, min_turnout_required);
                    } else if !approval_passing {
                        println!("                    • Approval rate too low: {:.2}% (threshold: {:.2}%)",
                                percent_approval, threshold_needed);
                    } else {
                        println!(
                            "                    • Voter turnout too low: {:.2}% (minimum: {:.2}%)",
                            turnout_percent, min_turnout_required
                        );
                    }
                }
            }
        } else {
            println!("Unexpected response format from reauth tally view");
        }
    }
}
