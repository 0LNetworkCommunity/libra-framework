use anyhow::Result;
use libra_config::extension::client_ext::ClientExt;
use libra_txs::{rest_client::Client, types::transaction::SignedTransaction};

pub async fn submit(signed_trans: &SignedTransaction) -> Result<()> {
    let client = Client::default()?;
    let pending_trans = client.submit(signed_trans).await?.into_inner();
    let res = client.wait_for_transaction(&pending_trans).await?;

    match res.inner().success() {
        true => {
          println!("transaction success!")
        },
        false => {
          println!("transaction not successful, status: {:?}", &res.inner().vm_status());
        },
    }
    Ok(())
}

/// Struct to organize all the TXS sending, so we're not creating new Client on every TX, if there are multiple.
pub struct Sender {
  client: Client,
  local_account: LocalAccount,
}

impl Sender {
  pub fn new(account_key: AccountKey) -> Self {
    let client = Client::default()?;

    // let legacy = libra_wallet::legacy::get_keys_from_prompt()?;

    // let owner = legacy.child_0_owner;
    let new_key = AccountKey::from_private_key(owner.pri_key);

    let seq = client.get_sequence_number(owner.account).await?;
    let mut local_account = LocalAccount::new(owner.account, new_key, seq);
    Self {
      client,
      local_acct
    }
  }

  pub fn refresh_sequence(&mut self){
    let seq = self.client.get_sequence_number(owner.account).await?;
    self.local_account.sequence = seq;

  }

  pub async fn submit(self, signed_trans: &SignedTransaction) -> Result<(VMStatus)> {
    let pending_trans = self.client.submit(signed_trans).await?.into_inner();
    let res = self.client.wait_for_transaction(&pending_trans).await?;

    let status = res.inner().vm_status();
    match res.inner().success() {
        true => {
          println!("transaction success!");
        
        },
        false => {
          println!("transaction not successful, status: {:?}", &status);
        },
    }
    Ok(status)
  }
}