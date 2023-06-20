use anyhow::Result;
use zapatos_sdk::{
  rest_client::Client,
  types::account_address::AccountAddress,
};
use libra_types::type_extensions::client_ext::ClientExt;
use serde::de::DeserializeOwned;

#[derive(Debug, clap::Subcommand)]
pub enum QueryType {
    /// Account balance
    Balance {
        #[clap(short, long)]
        /// account to query txs of
        account: AccountAddress,
    },
    // /// Unlocked Account balance
    // UnlockedBalance {
    //     #[clap(short, long)]
    //     /// account to query txs of
    //     account: AccountAddress,
    // },
    /// Epoch and waypoint
    Epoch,
    /// Network block height
    BlockHeight,
    /// All account resources
    Resources {
        #[clap(short, long)]
        /// account to query txs of
        account: AccountAddress,
    },
    /// get a move value from account blob
    MoveValue {
        #[clap(short, long)]
        /// account to query txs of
        account: AccountAddress,
        #[clap(long)]
        /// move module name
        module_name: String,
        #[clap(long)]
        /// move struct name
        struct_name: String,
        #[clap(long)]
        /// move key name
        key_name: String,
    },
    /// How far behind the local is from the upstream nodes
    SyncDelay,
    /// Get transaction history
    Txs {
        #[clap(short, long)]
        /// account to query txs of
        account: AccountAddress,
        #[clap(long)]
        /// get transactions after this height
        txs_height: Option<u64>,
        #[clap(long)]
        /// limit how many txs
        txs_count: Option<u64>,
        #[clap(long)]
        /// filter by type
        txs_type: Option<String>,
    },
    // /// Get events
    // Events {
    //     /// account to query events
    //     account: AccountAddress,
    //     /// switch for sent or received events.
    //     sent_or_received: bool,
    //     /// what event sequence number to start querying from, if DB does not have all.
    //     seq_start: Option<u64>,
    // },
    // /// get the validator's on-chain configuration, including network discovery addresses
    // ValConfig {
    //     /// the account of the validator
    //     account: AccountAddress,
    // },
    
    // TODO: View function
}

impl QueryType {

  pub async fn query(&self, client: Option<Client>) -> Result<Vec<serde_json::Value>>{
    let client = client.unwrap_or_else(|| Client::default().expect("Failed to create client"));

    // let client = client.unwrap_or_else(|| Client::default().expect("Failed to create client"));
    match self {
        QueryType::Balance { account } => {
          Ok(client.get_account_balance_libra(*account).await?)
        },
        _ => { todo!() }
        // QueryType::Epoch => todo!(),
        // QueryType::BlockHeight => todo!(),
        // QueryType::Resources { account } => todo!(),
        // QueryType::MoveValue { account, module_name, struct_name, key_name } => todo!(),
        // QueryType::SyncDelay => todo!(),
        // QueryType::Txs { account, txs_height, txs_count, txs_type } => todo!(),
    }

  }

  pub async fn query_into_type<T: DeserializeOwned> (&self, client: Option<Client>) -> Result<T> {
    let value = self.query(client).await?;
    if let Some(res) = value.into_iter().next() {
      Ok(serde_json::from_value::<T>(res)?)
    } else {
      Err(anyhow::anyhow!("no value found"))
    }
  }
}

// pub struct Querier {
//     pub client: Client,
// }

// impl Querier {
//     pub fn new(client: Client) -> Self {
//         Self { client }
//     }

//     pub async fn query(&self, query_type: QueryType) -> Result<Vec<serde_json::Value>> {
//         match query_type {
//             QueryType::Balance { account } => {
//               self.client.get_account_balance_libra(account).await
//             },
//             // QueryType::UnlockedBalance { account } => todo!(),
//             QueryType::Epoch => todo!(),
//             QueryType::BlockHeight => todo!(),
//             QueryType::Resources { account } => todo!(),
//             QueryType::MoveValue { account, module_name, struct_name, key_name } => todo!(),
//             QueryType::SyncDelay => todo!(),
//             QueryType::Txs { account, txs_height, txs_count, txs_type } => todo!(),
//             // QueryType::Events { account, sent_or_received, seq_start } => todo!(),
//             // QueryType::ValConfig { account } => todo!(),
//         }
//     }
// }
