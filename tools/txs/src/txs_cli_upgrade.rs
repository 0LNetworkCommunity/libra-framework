//! Validator subcommands

use crate::submit_transaction::Sender;
use std::path::PathBuf;

use zapatos_types::transaction::Script;
use zapatos_types::transaction::TransactionPayload;

use libra_cached_packages::aptos_stdlib::{
    aptos_governance_ol_create_proposal_v2, aptos_governance_ol_vote,
};

#[derive(clap::Subcommand)]
pub enum UpgradeTxs {
    /// after compiling a proposal script with `libra-framework upgrade` any authorized voter can create a proposal.
    Propose {
        #[clap(short, long)]
        /// string of the hash of the execution output of the upgrade script
        script_hash: String,

        #[clap(short, long)]
        /// a url which describes the proposal
        metadata_url: String,

        #[clap(short='d', long)]
        /// Path to the directory of the compiled proposal script
        script_dir: Option<PathBuf>,
    },
    Vote {
        #[clap(short, long)]
        /// the on chain ID of the proposal
        proposal_id: u64,
        #[clap(short, long, default_value = "true")]
        /// whether the proposal should pass (true), or be rejected (false)
        should_pass: bool,
    },
    /// All proposals need to be resolved by any user submitting the actual bytes in a transaction. This transaction has it's hash registered in the proposal, so that only the actual bytes of the script can be submitted, and any user is able to do so. This assumes that the proposal passed.
    Resolve {
        #[clap(short, long)]
        /// Path to the directory of the compiled proposal script
        proposal_script_dir: PathBuf,
    },
}

impl UpgradeTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = match self {
            UpgradeTxs::Propose {
                script_hash: hash, metadata_url, ..
            } => {
                aptos_governance_ol_create_proposal_v2(
                    hex::decode(hash)?,
                    metadata_url.as_bytes().to_vec(),
                    "metadata struct".to_string().as_bytes().to_vec(), // TODO
                    true,
                )
            }
            UpgradeTxs::Vote {
                proposal_id,
                should_pass,
            } => aptos_governance_ol_vote(*proposal_id, *should_pass),
            UpgradeTxs::Resolve {
                proposal_script_dir,
            } => {
                assert!(
                    &proposal_script_dir.exists(),
                    "proposal script cannot be found at {proposal_script_dir:?}"
                );

                // TODO: get the compiled script
                let proposal_bytes = std::fs::read(proposal_script_dir).unwrap();

                let proposal_script = Script::new(proposal_bytes, vec![], vec![]);

                TransactionPayload::Script(proposal_script)
            }
        };

        sender.sign_submit_wait(payload).await?;
        Ok(())
    }
}
