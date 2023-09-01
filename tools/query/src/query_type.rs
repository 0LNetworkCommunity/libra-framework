use crate::{
    account_queries::{get_account_balance_libra, get_tower_state},
    query_view::get_view,
};
use anyhow::{bail, Result};
use indoc::indoc;
use libra_types::exports::AuthenticationKey;
use libra_types::type_extensions::client_ext::ClientExt;
use serde_json::json;
use diem_sdk::{rest_client::Client, types::account_address::AccountAddress};

#[derive(Debug, clap::Subcommand)]
pub enum QueryType {
    /// Account balance
    Balance {
        #[clap(short, long)]
        /// account to query txs of
        account: AccountAddress,
    },
    /// User's Tower state
    Tower {
        #[clap(short, long)]
        /// account to query txs of
        account: AccountAddress,
    },
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
    /// Execute a View function on-chain
    View {
        #[clap(
            short,
            long,
            help = indoc!{r#"
                Function identifier has the form <ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>

                Example:
                0x1::coin::balance
            "#}
        )]
        function_id: String,

        #[clap(
            short,
            long,
            help = indoc!{r#"
                Type arguments separated by commas

                Example:
                'u8, u16, u32, u64, u128, u256, bool, address, vector<u8>, signer'
            "#}
        )]
        type_args: Option<String>,

        #[clap(
            short,
            long,
            help = indoc!{r#"
                Function arguments separated by commas

                Example:
                '0x1, true, 12, 24_u8, x"123456"'
            "#}
        )]
        args: Option<String>,
    },
    /// Looks up the address of an account given an auth key. The authkey diverges from the address after a key rotation.
    LookupAddress {
        #[clap(short, long)]
        auth_key: AuthenticationKey, // we use account address to parse, because that's the format needed to lookup users. AuthKeys and AccountAddress are the same formats.
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
}

impl QueryType {
    pub async fn query_to_json(&self, client_opt: Option<Client>) -> Result<serde_json::Value> {
        let client = match client_opt {
            Some(c) => c,
            None => Client::default().await?,
        };

        match self {
        QueryType::Balance { account } => {
          let res = get_account_balance_libra(&client, *account).await?;
          Ok(json!(res.scaled()))
        },
        QueryType::Tower { account } => {
          let res = get_tower_state(&client, *account).await?;
          Ok(json!(res))

        },
        QueryType::View {
            function_id,
            type_args,
            args,
        } => {
            let res = get_view(&client, function_id, type_args.to_owned(), args.to_owned()).await?;
            let json = json!({
              "body": res
            });
            Ok(json)
         },
        QueryType::Epoch => {
            let res = get_view(&client, "0x1::reconfiguration::get_current_epoch", None, None).await?;

            let num: Vec<String> = serde_json::from_value(res)?;
            let json = json!({
              "epoch": num.first().unwrap().parse::<u64>()?,
            });
            Ok(json)
        },
        QueryType::LookupAddress { auth_key } => {
          let addr = client.lookup_originating_address( auth_key.to_owned()).await?;

          Ok(json!({
            "address": addr
          }))
        },
        _ => { bail!("Not implemented for type: {:?}", self) }
        // QueryType::BlockHeight => todo!(),
        // QueryType::Resources { account } => todo!(),
        // QueryType::MoveValue { account, module_name, struct_name, key_name } => todo!(),

    }
    }
}
