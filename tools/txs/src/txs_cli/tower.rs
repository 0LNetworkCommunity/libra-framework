use super::submit_transaction::Sender;
use anyhow::{Result};
use zapatos_types::account_address::AccountAddress;
use libra_cached_packages::aptos_framework_sdk_builder::EntryFunctionCall::{ TowerStateMinerstateCommit};

impl Sender {
  pub async fn tower_submit(&mut self, 
    challenge: Vec<u8>,
    solution:  Vec<u8>,
    difficulty: u64,
    security: u64,
  ) -> anyhow::Result<()> {

    let payload = TowerStateMinerstateCommit {
      challenge,
      solution,
      difficulty,
      security,
    }.encode();

    self.sign_submit_wait(payload).await?;
    Ok(())
  }

}
