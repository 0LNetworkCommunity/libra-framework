
use anyhow::{bail};
use zapatos_sdk::rest_client::Client;
use zapatos_types::transaction::SignedTransaction;
use zapatos_sdk::types::chain_id::ChainId;
use zapatos_sdk::types::{LocalAccount, AccountKey};
use zapatos_sdk::transaction_builder::TransactionBuilder;
use zapatos::account::key_rotation::lookup_address;
use zapatos_types::transaction::TransactionPayload;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use libra_types::type_extensions::client_ext::{ClientExt, DEFAULT_TIMEOUT_SECS};

// #[derive(Debug)]
// /// a transaction error type specific to ol txs
// pub struct TxError {
//     /// the actual error type
//     pub err: Option<Error>,
//     /// transaction view if the transaction got that far
//     pub tx_view: Option<TransactionView>,
//     /// Move module or script where error occurred
//     pub location: Option<String>,
//     /// Move abort code used in error
//     pub abort_code: Option<u64>,
// }

// impl From<anyhow::Error> for TxError {
//     fn from(e: anyhow::Error) -> Self {
//         TxError {
//             err: Some(e),
//             tx_view: None,
//             location: None,
//             abort_code: None,
//         }
//     }
// }

// pub async fn submit(signed_trans: &SignedTransaction) -> anyhow::Result<String> {
//     let client = Client::default()?;
//     let pending_trans = client.submit(signed_trans).await?.into_inner();
//     let res = client.wait_for_transaction(&pending_trans).await?;

//     match res.inner().success() {
//         true => {
//           // TODO: use logger crate
//           println!("transaction success!")
//           return Ok(res.inner().vm_status())
//         },
//         false => {
//           println!("transaction not successful, status: {:?}", &res.inner().vm_status());
//           return bail!("transaction not successful, status: {:?}", &res.inner().vm_status());
//         },
//     }
//     Ok(())
// }

/// Struct to organize all the TXS sending, so we're not creating new Client on every TX, if there are multiple.
pub struct Sender {
  client: Client,
  // address: AccountAddress,
  local_account: LocalAccount,
  chain_id: ChainId,
}

impl Sender {
  pub async fn new(account_key: AccountKey, chain_id: ChainId, client_opt: Option<Client>) -> anyhow::Result<Self> {
    let client = client_opt.unwrap_or(Client::default()?);

    let address = lookup_address(&client, account_key.authentication_key().derived_address(), false).await?;

    let seq = client.get_sequence_number(address).await?;
    let local_account = LocalAccount::new(address, account_key, seq);


    Ok(Self {
      client,
      // address,
      local_account,
      chain_id,
    })
  }

  pub async fn sign_submit_wait(&mut self, payload: TransactionPayload) -> anyhow::Result<String> {
    let signed = self.sign_payload(payload);
    let r = self.submit(&signed).await?;
    Ok(r)
  }

  pub fn sign_payload(&mut self, payload: TransactionPayload) -> SignedTransaction {
      let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
      let time = t + DEFAULT_TIMEOUT_SECS*10;
      let tb = TransactionBuilder::new(payload, time, self.chain_id);
      self.local_account.sign_with_transaction_builder(tb)
  }
  // pub fn refresh_sequence(&mut self) -> Result<()>{
  //   // let seq = self.client.get_sequence_number(self.local_account.address).await?;
  //   let s = self.local_account.sequence_number_mut();
  //   s = self.client.get_sequence_number(self.local_account.address).await?
  //   Ok(())
  // }

  // TODO: return a more useful type, 
  pub async fn submit(&self, signed_trans: &SignedTransaction) -> anyhow::Result<String>{
    // let client = Client::default()?;
    let pending_trans = self.client.submit(signed_trans).await?.into_inner();
    let res = self.client.wait_for_transaction(&pending_trans).await?;

    match res.inner().success() {
        true => {
          println!("transaction success!");
          Ok(res.inner().vm_status())
        },
        false => {
          println!("transaction not successful, status: {:?}", &res.inner().vm_status());
          bail!("transaction not successful, status: {:?}", &res.inner().vm_status())
        },
    }
}
}