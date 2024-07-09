//! Validator subcommands

use crate::submit_transaction::Sender;
use anyhow::{bail, Context};
use diem_sdk::types::transaction::TransactionArgument;
use diem_types::transaction::{Script, TransactionPayload};
use libra_cached_packages::{
    libra_stdlib,
    libra_stdlib::{diem_governance_ol_create_proposal_v2, diem_governance_ol_vote},
};
use std::{fs, path::PathBuf};

#[derive(clap::Subcommand)]
pub enum GovernanceTxs {
    /// Register an upgrade proposal. NOTE: requires compiling a proposal script
    Propose {
        #[clap(short = 'd', long)]
        /// Path to the directory of the compiled proposal script
        proposal_script_dir: PathBuf,

        #[clap(short, long)]
        /// URL which describes the proposal
        metadata_url: String,
    },
    /// Vote for a chain upgrade
    Vote {
        #[clap(short = 'i', long)]
        /// On-chain ID of the proposal
        proposal_id: u64,

        #[clap(long)]
        /// Must explicitly inform if it should fail (default is to pass)
        // In the absense of this flag it assumes you are voting "should pass"
        should_fail: bool,
    },
    /// Resolve by sending all compiled bytes of the transaction
    // All proposals need to be resolved by any user submitting the actual bytes in a transaction. This transaction has it's hash registered in the proposal, so that only the actual bytes of the script can be submitted, and any user is able to do so. This assumes that the proposal passed.
    Resolve {
        #[clap(short = 'i', long)]
        /// On-chain ID of the proposal
        proposal_id: u64,
        #[clap(short = 'd', long)]
        /// Path to the directory of the compiled proposal script
        proposal_script_dir: PathBuf,
    },
    /// Tickle the epoch boundary
    EpochBoundary,
}

impl GovernanceTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = match self {
            GovernanceTxs::Propose {
                proposal_script_dir,
                metadata_url,
            } => {
                let hash_path = proposal_script_dir.join("script_sha3");
                if !proposal_script_dir.exists() || !hash_path.exists() {
                    bail!(
                        "cannot find upgrade script package at {:?}",
                        proposal_script_dir
                    );
                }
                let hash = fs::read_to_string(&hash_path)?;

                let num =
                    libra_query::chain_queries::get_next_governance_proposal_id(sender.client())
                        .await?;

                println!(
                    "next proposal id is: {}. Save this and use it for voting.",
                    &num
                );

                diem_governance_ol_create_proposal_v2(
                    hex::decode(hash)?,
                    metadata_url.as_bytes().to_vec(),
                    "metadata struct".to_string().as_bytes().to_vec(), // TODO
                    true,
                )
            }
            GovernanceTxs::Vote {
                proposal_id,
                should_fail,
            } => diem_governance_ol_vote(*proposal_id, !*should_fail), // NOTE: the default is to vote for "pass" so we invert the BOOL versus what diem_governance.move expects.
            GovernanceTxs::Resolve {
                proposal_id,
                proposal_script_dir,
            } => {
                if libra_query::chain_queries::is_gov_proposal_resolved(
                    sender.client(),
                    *proposal_id,
                )
                .await
                .context("cannot get status of proposal")?
                {
                    bail!("proposal {} has already been resolved", proposal_id);
                }

                assert!(
                    &proposal_script_dir.exists(),
                    "proposal script cannot be found at {proposal_script_dir:?}"
                );

                let proposal_bytes = std::fs::read(proposal_script_dir.join("script.mv")).unwrap();

                let proposal_script = Script::new(
                    proposal_bytes,
                    vec![],
                    vec![
                        TransactionArgument::U64(*proposal_id), // proposal_id.into()
                    ],
                );

                TransactionPayload::Script(proposal_script)
            }
            GovernanceTxs::EpochBoundary => libra_stdlib::diem_governance_trigger_epoch(),
        };

        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}
