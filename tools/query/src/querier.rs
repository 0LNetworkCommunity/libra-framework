use anyhow::Result;
use zapatos_sdk::{
  rest_client::{Client, Response},
  types::account_address::AccountAddress,
};

#[derive(Debug, clap::Subcommand)]
pub enum QueryType {
    /// Account balance
    Balance {
        /// account to query txs of
        account: AccountAddress,
    },
    /// Unlocked Account balance
    UnlockedBalance {
        /// account to query txs of
        account: AccountAddress,
    },
    /// Epoch and waypoint
    Epoch,
    /// Network block height
    BlockHeight,
    /// All account resources
    Resources {
        /// account to query txs of
        account: AccountAddress,
    },
    /// get a move value from account blob
    MoveValue {
        /// account to query txs of
        account: AccountAddress,
        /// move module name
        module_name: String,
        /// move struct name
        struct_name: String,
        /// move key name
        key_name: String,
    },
    /// How far behind the local is from the upstream nodes
    SyncDelay,
    /// Get transaction history
    Txs {
        /// account to query txs of
        account: AccountAddress,
        /// get transactions after this height
        txs_height: Option<u64>,
        /// limit how many txs
        txs_count: Option<u64>,
        /// filter by type
        txs_type: Option<String>,
    },
    /// Get events
    Events {
        /// account to query events
        account: AccountAddress,
        /// switch for sent or received events.
        sent_or_received: bool,
        /// what event sequence number to start querying from, if DB does not have all.
        seq_start: Option<u64>,
    },
    /// get the validator's on-chain configuration, including network discovery addresses
    ValConfig {
        /// the account of the validator
        account: AccountAddress,
    },
    
    // TODO: View function
}

pub struct Querier {
    pub client: Client,
}

impl Querier {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn query(&self, query_type: QueryType) -> Result<Response> {
        match query_type {
            QueryType::Balance { account } => todo!(),
            QueryType::UnlockedBalance { account } => todo!(),
            QueryType::Epoch => todo!(),
            QueryType::BlockHeight => todo!(),
            QueryType::Resources { account } => todo!(),
            QueryType::MoveValue { account, module_name, struct_name, key_name } => todo!(),
            QueryType::SyncDelay => todo!(),
            QueryType::Txs { account, txs_height, txs_count, txs_type } => todo!(),
            QueryType::Events { account, sent_or_received, seq_start } => todo!(),
            QueryType::ValConfig { account } => todo!(),
        }
    }
}
