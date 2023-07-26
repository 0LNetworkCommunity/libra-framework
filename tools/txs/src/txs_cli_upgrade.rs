//! Validator subcommands

use crate::submit_transaction::Sender;
use std::fs;
use std::path::PathBuf;

use anyhow::bail;
use anyhow::Context;
use zapatos_types::transaction::Script;
use zapatos_types::transaction::TransactionPayload;

use libra_cached_packages::aptos_stdlib::{
    aptos_governance_ol_create_proposal_v2, aptos_governance_ol_vote,
};

#[derive(clap::Subcommand)]
pub enum UpgradeTxs {
    /// after compiling a proposal script with `libra-framework upgrade` any authorized voter can create a proposal.
    Propose {
        #[clap(short = 'd', long)]
        /// Path to the directory of the compiled proposal script
        proposal_script_dir: PathBuf,

        #[clap(short, long)]
        /// a url which describes the proposal
        metadata_url: String,
    },
    Vote {
        #[clap(short='i', long)]
        /// the on chain ID of the proposal
        proposal_id: u64,

        #[clap(long)]
        /// must explicitly inform if it should fail. In the absense of this flag it assumes you are voting "should pass"
        should_fail: bool,
    },
    /// All proposals need to be resolved by any user submitting the actual bytes in a transaction. This transaction has it's hash registered in the proposal, so that only the actual bytes of the script can be submitted, and any user is able to do so. This assumes that the proposal passed.
    Resolve {
        #[clap(short='i', long)]
        /// the on chain ID of the proposal
        proposal_id: u64,
        #[clap(short='d', long)]
        /// Path to the directory of the compiled proposal script
        proposal_script_dir: PathBuf,
    },
}

impl UpgradeTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = match self {
            UpgradeTxs::Propose {
                proposal_script_dir,
                metadata_url,
            } => {
                let hash_path = proposal_script_dir.join("script_sha3");
                if !proposal_script_dir.exists() || !hash_path.exists() {
                    bail!(
                        "cannot find upgrade script pacage at {:?}",
                        proposal_script_dir
                    );
                }
                let hash = fs::read_to_string(&hash_path)?;

                let num = libra_query::chain_queries::get_next_governance_proposal_id(sender.client()).await?;
                // let query_res = libra_query::query_view::run(
                //     "0x1::aptos_governance::get_next_governance_proposal_id",
                //     None,
                //     None,
                // )
                // .await?;
                // // let id: Vec<String> = serde_json::from_value(query_res)?;
                // let num: u64 = serde_json::from_value::<Vec<String>>(query_res)?
                //   .iter()
                //   .next()
                //   .context("could not get a response from view function get_next_governance_proposal_id")?
                //   .parse()?;

                println!("next proposal id is: {}. Save this and use it for voting.", &num);

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
                proposal_id, proposal_script_dir,
            } => {

                if libra_query::chain_queries::is_gov_proposal_resolved(sender.client(), *proposal_id).await.context("cannot get status of proposal")? {
                  bail!("proposal {} has already been resolved", proposal_id);
                }

                if !libra_query::chain_queries::can_gov_proposal_resolve(sender.client(), *proposal_id).await.context("proposal cannot be resolved")?{
                  bail!("proposal {} is not resolvable", proposal_id);
                }
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
