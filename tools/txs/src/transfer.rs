//! form a transfer payload and execute transaction
use zapatos_sdk::types::account_address::AccountAddress;
use libra_cached_packages::aptos_framework_sdk_builder::EntryFunctionCall:: OlAccountTransfer;
use super::submit_transaction::Sender;

impl Sender {

  pub async fn transfer(&mut self, to: AccountAddress, amount: u64) -> anyhow::Result<()> {

      let payload = OlAccountTransfer {
        to,
        amount,
      }.encode();

      self.sign_submit_wait(payload).await?;
      Ok(())
  }
}
