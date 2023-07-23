//! Validator subcommands

use crate::submit_transaction::Sender;
use libra_cached_packages::aptos_stdlib::EntryFunctionCall::{
  JailUnjailByVoucher,
  ProofOfFeePofRetractBid,
  ProofOfFeePofUpdateBid,
  VouchRevoke,
  VouchVouchFor,
};
use zapatos_types::account_address::AccountAddress;

#[derive(clap::Subcommand)]
pub enum ValidatorTxs {
    Pof {
        #[clap(short, long)]
        /// what percent of the nominal reward you are paying to join set, with two decimal places: 12.34 becomes 12.34%
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
        vouch_acct: AccountAddress,
        #[clap(short, long)]
        /// If you are revoking the vouch for the account specified here.
        revoke: bool,
    },
}

impl ValidatorTxs {
    pub async fn run(&self, sender: &mut Sender) -> anyhow::Result<()> {
        let payload = match self {
            ValidatorTxs::Pof { bid_pct, expiry: epoch_expiry , retract} => {
              if *retract {
                ProofOfFeePofRetractBid{}
              } else {
                ProofOfFeePofUpdateBid{
                    bid: (*bid_pct * 100.00) as u64, // scale two decimal places
                    epoch_expiry: *epoch_expiry,
                }
              }
            }
            ValidatorTxs::Jail { unjail_acct } => {
              JailUnjailByVoucher { addr: unjail_acct.to_owned() }
            }
            ValidatorTxs::Vouch { vouch_acct, revoke } => {
              if *revoke {
                VouchRevoke {
                  its_not_me_its_you: *vouch_acct
                }

              } else {
                VouchVouchFor {
                  wanna_be_my_friend: *vouch_acct
                }
              }
            },
        };

        sender.sign_submit_wait(payload.encode()).await?;
        Ok(())
    }
}
