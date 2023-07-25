//! Validator subcommands

use crate::submit_transaction::Sender;
use std::fs;
use std::path::PathBuf;

use anyhow::bail;
use zapatos_types::transaction::Script;
use zapatos_types::transaction::TransactionPayload;

use libra_cached_packages::aptos_stdlib::{
    aptos_governance_ol_create_proposal_v2, aptos_governance_ol_vote,
};

#[derive(clap::Subcommand)]
pub enum UpgradeTxs {
    /// after compiling a proposal script with `libra-framework upgrade` any authorized voter can create a proposal.
    Propose {

        #[clap(short='d', long)]
        /// Path to the directory of the compiled proposal script
        proposal_script_dir: PathBuf,

        #[clap(short, long)]
        /// a url which describes the proposal
        metadata_url: String,

    },
    Vote {
        #[clap(short, long)]
        /// the on chain ID of the proposal
        proposal_id: u64,

        #[clap(short, long)]
        /// must explicitly inform if it should fail. In the absense of this flag it assumes you are voting "should pass"
        should_fail: bool,
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
                proposal_script_dir, metadata_url,
            } => {
                let hash_path = proposal_script_dir.join("script_sha3");
                if !proposal_script_dir.exists() || !hash_path.exists() {
                  bail!("cannot find upgrade script pacage at {:?}", proposal_script_dir);
                }
                let hash = fs::read_to_string(&hash_path)?;

                aptos_governance_ol_create_proposal_v2(
                    hex::decode(hash)?,
                    metadata_url.as_bytes().to_vec(),
                    "metadata struct".to_string().as_bytes().to_vec(), // TODO
                    true,
                )
            }
            UpgradeTxs::Vote {
                proposal_id,
                should_fail,
            } => aptos_governance_ol_vote(*proposal_id, !*should_fail), // NOTE: we are inverting the BOOL here.
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
