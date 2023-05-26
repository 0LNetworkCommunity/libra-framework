use crate::extension::client_ext::ClientExt;
use anyhow::Result;
use zapatos_sdk::{
    coin_client::CoinClient, rest_client::Client, types::account_address::AccountAddress,
};
use QueryType::*;

#[derive(Debug)]
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
}

pub struct Querier {
    pub client: Client,
}

impl Querier {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn query(&self, query_type: QueryType) -> Result<String> {
        let print = match query_type {
            Balance { account } => {
                let coin_client = CoinClient::new(&self.client);
                coin_client.get_account_balance(&account).await?.to_string()
            }
            Resources { account } => self.client.get_account_resources_ext(account).await?,
            _ => {
                //TODO: Implement other types of Query
                String::new()
            }
        };
        Ok(print)
    }
}
