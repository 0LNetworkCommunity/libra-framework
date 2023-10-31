//! Validator subcommands

use std::{fs, path::PathBuf};

use crate::submit_transaction::Sender;
use anyhow::{bail, Context};
use diem_genesis::config::OperatorConfiguration;
use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib::EntryFunctionCall::{
    JailUnjailByVoucher, ProofOfFeePofRetractBid, ProofOfFeePofUpdateBid,
    StakeUpdateNetworkAndFullnodeAddresses, ValidatorUniverseRegisterValidator, VouchRevoke,
    VouchVouchFor,
};

use libra_types::global_config_dir;
use libra_wallet::validator_files::OPERATOR_FILE;

#[derive(clap::Subcommand)]
pub enum ValidatorTxs {
    Pof {
        #[clap(short, long)]
        /// the percentage of the nominal reward you will bid to join the validator set. Numbers can include three decimal places: 1.234 is 123.4%. Note this is the maximum precision allowed in the bid (i.e. one decimal of a percent). Numbers with more decimals will be truncated (not rounded)
        bid_pct: f64,
        #[clap(short, long)]
        /// epoch number in which this bid expires. The validity of the bit is inclusive of the epoch number, that is, on `expiry + 1` the bid is no longer valid. If it should not expire the number must be 0.
        expiry: u64,
        #[clap(short, long)]
        /// eliminates the bid. There are only a limited amount of retractions that can happen in an epoch.
        retract: bool,
    },
    Jail {
        #[clap(short, long)]
        /// you are a voucher for a validator which is jailed. you are un-jailing this validator after checking that they are able to join again.
        unjail_acct: AccountAddress,
    },
    Vouch {
        #[clap(short, long)]
        /// This is an account that you are vouching for. They may not be a validator account.
        vouch_for: AccountAddress,
        #[clap(short, long)]
        /// If you are revoking the vouch for the account specified here.
        revoke: bool,
    },
    Register {
        #[clap(short('f'), long)]
        /// optional, Path to files with registration files
        operator_file: Option<PathBuf>,
    },
    Update {
        #[clap(short('f'), long)]
        /// optional, Path to files with registration files
        operator_file: Option<PathBuf>,
    },
}

impl ValidatorTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = match self {
            ValidatorTxs::Pof {
                bid_pct,
                expiry: epoch_expiry,
                retract,
            } => {
                if *retract {
                    ProofOfFeePofRetractBid {}
                } else {
                    // TODO: the u64 will truncate, but without rounding it will drop the last digit.
                    let scaled_bid = (bid_pct * 1000.0).round() as u64; // scale to 10ˆ3.
                    if scaled_bid > 1100 {
                        bail!(
                            "a bid amount at 110.0% or above the epoch's reward, will be rejected"
                        );
                    }
                    ProofOfFeePofUpdateBid {
                        bid: scaled_bid,
                        epoch_expiry: *epoch_expiry,
                    }
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
                        its_not_me_its_you: *vouch_acct,
                    }
                } else {
                    VouchVouchFor {
                        wanna_be_my_friend: *vouch_acct,
                    }
                }
            }
            ValidatorTxs::Register { operator_file } => {
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
                let vfn_fullnode_protocol =
                    fullnode_host.as_network_address(oc.full_node_network_public_key.context(
                        "cannot find fullnode network public key in operator config file",
                    )?)?;

                ValidatorUniverseRegisterValidator {
                    consensus_pubkey: oc.consensus_public_key.to_bytes().to_vec(),
                    proof_of_possession: oc.consensus_proof_of_possession.to_bytes().to_vec(),
                    network_addresses: bcs::to_bytes(&val_net_protocol)?,
                    fullnode_addresses: bcs::to_bytes(&vfn_fullnode_protocol)?,
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
                    new_network_addresses: bcs::to_bytes(&val_net_protocol)?,
                    new_fullnode_addresses: bcs::to_bytes(&vfn_fullnode_protocol)?,
                }
            }
        };

        sender.sign_submit_wait(payload.encode()).await?;
        Ok(())
    }
}
