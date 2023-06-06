use anyhow::Result;
use libra_config::extension::client_ext::ClientExt;
use libra_txs::{rest_client::Client, types::transaction::SignedTransaction};
use libra_txs::extension::client_ext::ClientExt as ClientExtDupl;
use zapatos_sdk::types::{LocalAccount, AccountKey};
use zapatos_sdk::types::vm_status::VMStatus;
use zapatos_sdk::types::account_address::AccountAddress;

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
  address: AccountAddress,
  local_account: LocalAccount,
}

impl Sender {
  pub async fn new(address: AccountAddress, account_key: AccountKey) -> anyhow::Result<Self> {
    let client = Client::default()?;

    // let legacy = libra_wallet::legacy::get_keys_from_prompt()?;

    // let owner = legacy.child_0_owner;
    // let new_key = AccountKey::from_private_key(account_key.private_key());

    let seq = client.get_sequence_number(address).await?;
    let mut local_account = LocalAccount::new(address, account_key, seq);
    Ok(Self {
      client,
      address,
      local_account,
    })
  }

  // pub fn refresh_sequence(&mut self) -> Result<()>{
  //   // let seq = self.client.get_sequence_number(self.local_account.address).await?;
  //   let s = self.local_account.sequence_number_mut();
  //   s = self.client.get_sequence_number(self.local_account.address).await?
  //   Ok(())
  // }

  pub async fn submit(self, signed_trans: &SignedTransaction) -> Result<VMStatus, VMStatus> {
    let pending_trans = self.client.submit(signed_trans).await?.into_inner();
    let res = self.client.wait_for_transaction(&pending_trans).await?;

    let status = res.inner().vm_status();
    match res.inner().success() {
        true => {
          println!("transaction success!");
          return Ok(status)
        
        },
        false => {
          println!("transaction not successful, status: {:?}", &status);
          return Err(status)
        },
    }
    
  }
}