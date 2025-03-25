//! Validator subcommands

use crate::submit_transaction::Sender;
use anyhow::{bail, Context};
use diem_genesis::config::OperatorConfiguration;
use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib::EntryFunctionCall::{
    self, JailUnjailByVoucher, ProofOfFeePofRetractBid, ProofOfFeePofUpdateBid,
    ProofOfFeePofUpdateBidNetReward, StakeUpdateNetworkAndFullnodeAddresses,
    ValidatorUniverseRegisterValidator, VouchRevoke, VouchVouchFor,
};
use libra_config::validator_registration;
use libra_types::global_config_dir;
use libra_wallet::validator_files::OPERATOR_FILE;
use std::{fs, path::PathBuf};

#[derive(clap::Subcommand)]
pub enum ValidatorTxs {
    /// Proof-of-Fee auction bidding
    Pof {
        #[clap(short, long)]
        /// Percentage of the nominal reward you will bid to join the
        /// validator set, with three decimal places: 1.234 is 123.4%
        bid_pct: Option<f64>,

        #[clap(short, long)]
        /// Epoch until the bid is valid (will expire in `expiry` + 1)
        epoch_expiry: u64,
        #[clap(short, long)]
        /// Eliminates the bid. There are only a limited amount of retractions that can happen in an epoch
        retract: bool,

        #[clap(short('r'), long, conflicts_with = "bid_pct")]
        /// experimental. Estimated net reward you would like to receive each epoch
        net_reward: Option<u64>,
    },
    /// Jail and unjail transactions
    Jail {
        #[clap(short, long)]
        /// Un-jail this validator. Used by any validators which are vouching for a validator which is jailed
        unjail_acct: AccountAddress,
    },
    /// Vouch for accounts
    Vouch {
        #[clap(short, long)]
        /// Vouch for another account, usually for validators
        vouch_for: AccountAddress,
        #[clap(short, long)]
        /// Revoke a vouch for an account
        revoke: bool,
    },
    /// Register as a validator
    Register {
        #[clap(short('f'), long)]
        /// optional, Path to files with registration files
        operator_file: Option<PathBuf>,
    },
    /// Update validator configurations
    Update {
        #[clap(short('f'), long)]
        /// optional, Path to files with registration files
        operator_file: Option<PathBuf>,
    },
}

impl ValidatorTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = self.make_payload()?;
        sender.sign_submit_wait(payload.encode()).await?;
        Ok(())
    }

    //  Create the Entry function which the txs will run.
    pub fn make_payload(&self) -> anyhow::Result<EntryFunctionCall> {
        let p = match self {
            ValidatorTxs::Pof {
                net_reward,
                bid_pct,
                epoch_expiry,
                retract,
            } => {
                if *retract {
                    ProofOfFeePofRetractBid {}
                } else if let Some(b) = bid_pct {
                    // TODO: the u64 will truncate, but without rounding it will drop the last digit.
                    let scaled_bid = (b * 1000.0).round() as u64; // scale to 10ˆ3.
                    if scaled_bid > 1100 {
                        bail!(
                            "a bid amount at 110.0% or above the epoch's reward, will be rejected"
                        );
                    }
                    ProofOfFeePofUpdateBid {
                        bid: scaled_bid,
                        epoch_expiry: *epoch_expiry,
                    }
                } else if let Some(n) = net_reward {
                    // Default path is to update based on the expected net reward
                    ProofOfFeePofUpdateBidNetReward {
                        net_reward: *n,
                        epoch_expiry: *epoch_expiry,
                    }
                } else {
                    bail!("either bid_pct or net_reward must be provided")
                }
            }
            ValidatorTxs::Jail { unjail_acct } => JailUnjailByVoucher {
                addr: unjail_acct.to_owned(),
            },
            ValidatorTxs::Vouch {
                vouch_for: vouch_acct,
                revoke,
            } => {
                if *revoke {
                    VouchRevoke {
                        friend_account: *vouch_acct,
                    }
                } else {
                    VouchVouchFor {
                        friend_account: *vouch_acct,
                    }
                }
            }
            ValidatorTxs::Register { operator_file } => {
                let reg = validator_registration::registration_from_operator_yaml(
                    operator_file.to_owned(),
                )?;
                ValidatorUniverseRegisterValidator {
                    consensus_pubkey: reg.consensus_pubkey,
                    proof_of_possession: reg.proof_of_possession,
                    network_addresses: reg.network_addresses,
                    fullnode_addresses: reg.fullnode_addresses,
                }
            }
            ValidatorTxs::Update { operator_file } => {
                let file = operator_file.to_owned().unwrap_or_else(|| {
                    let a = global_config_dir();
                    a.join(OPERATOR_FILE)
                });

                let yaml_str = fs::read_to_string(file)?;

                let oc: OperatorConfiguration = serde_yaml::from_str(&yaml_str)?;

                let val_net_protocol = oc
                    .validator_host
                    .as_network_address(oc.validator_network_public_key)?;

                let fullnode_host = oc
                    .full_node_host
                    .context("cannot find fullnode host in operator config file")?;
                let vfn_fullnode_protocol = fullnode_host
                    .as_network_address(oc.full_node_network_public_key.context(
                        "cannot find fullnode network public key operator config file",
                    )?)?;

                StakeUpdateNetworkAndFullnodeAddresses {
                    validator_address: oc.operator_account_address.into(),
                    new_network_addresses: bcs::to_bytes(&vec![val_net_protocol])?,
                    new_fullnode_addresses: bcs::to_bytes(&vec![vfn_fullnode_protocol])?,
                }
            }
        };

        Ok(p)
    }
}
