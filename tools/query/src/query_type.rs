use crate::{
    account_queries::{get_account_balance_libra, get_tower_state},
    query_view::fetch_and_display,
    query_move_value::get_account_move_value,
};
use anyhow::Result;
use diem_sdk::{rest_client::Client, types::account_address::AccountAddress};
use indoc::indoc;
use serde_json::json;
use libra_types::exports::AuthenticationKey;
use libra_types::legacy_types::app_cfg::AppCfg;
use libra_types::type_extensions::client_ext::ClientExt;
use libra_types::global_config_dir;

pub enum OutputType {
    Json(String),
    KeyValue(String),
}

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
        #[clap(
            short = 'a',
            long = "account",
            help = "Account to query transactions for"
        )]
        account: AccountAddress,
        #[clap(
            short = 's',
            long = "start",
            help = "Starting sequence number for transactions"
        )]
        start: Option<u64>,
        #[clap(
            short = 'l',
            long = "limit",
            help = "Limit the number of transactions to fetch"
        )]
        limit: Option<u64>,
    },
    /// Get events
    Events {
        /// Account to query events
        #[clap(short, long)]
        account: AccountAddress,
        /// Move module struct tag
        #[clap(
            short = 't',
            long = "tag",
            help = "Move module struct tag eg 0x1::stake::StakePool"
        )]
        struct_tag: String,
        /// Field name in the struct
        #[clap(
            short = 'f',
            long = "field",
            help = "Field name in the struct eg join_validator_set_events"
        )]
        field_name: String,
        /// Starting sequence number for events
        #[clap(
            short = 's',
            long = "start",
            help = "Starting sequence number for events"
        )]
        start: Option<u64>,
        /// Limit the number of events to fetch
        #[clap(
            short = 'l',
            long = "limit",
            help = "Limit the number of events to fetch"
        )]
        limit: Option<u16>,
    }, // /// get the validator's on-chain configuration, including network discovery addresses
       // ValConfig {
       //     /// the account of the validator
       //     account: AccountAddress,
       // },
}

impl QueryType {
    pub async fn query(&self, client_opt: Option<Client>) -> Result<OutputType> {
        let client = match client_opt {
            Some(c) => c,
            None => Client::default().await?,
        };

        match self {
            QueryType::Balance { account } => {
                let json_data = get_account_balance_libra(&client, *account).await?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
            QueryType::Tower { account } => {
                let tower_state = get_tower_state(&client, *account).await?;
                let json_data = serde_json::to_value(tower_state)?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
            QueryType::View {
                function_id,
                type_args,
                args,
            } => {
                let json_data =
                    fetch_and_display(&client, function_id, type_args.to_owned(), args.to_owned())
                        .await?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
            QueryType::Epoch => {
                let json_data = fetch_and_display(
                    &client,
                    "0x1::reconfiguration::get_current_epoch",
                    None,
                    None,
                )
                .await?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
            QueryType::BlockHeight => {
                let json_data =
                    fetch_and_display(&client, "0x1::block::get_current_block_height", None, None)
                        .await?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
            QueryType::Resources { account } => {
                let res = client
                    .get_account_resources(*account)
                    .await?
                    .into_inner()
                    .into_iter()
                    .map(|resource| {
                        let mut map = serde_json::Map::new();
                        map.insert(resource.resource_type.to_string(), resource.data);
                        serde_json::Value::Object(map)
                    })
                    .collect::<Vec<serde_json::Value>>();
                Ok(OutputType::Json(serde_json::to_string_pretty(&res)?))
            }
            QueryType::LookupAddress { auth_key } => {
                let addr = client.lookup_originating_address(auth_key.to_owned()).await?;

                Ok(OutputType::Json(serde_json::to_string(&json!({
                    "address": addr
                }))?))
            }
            QueryType::MoveValue {
                account,
                module_name,
                struct_name,
                key_name,
            } => {
                get_account_move_value(&client, &account, &module_name, &struct_name, &key_name).await
            }
            QueryType::SyncDelay {} => {
                let config_path = global_config_dir();
                let mut app_cfg = AppCfg::load(Some(config_path))?;

                // Get the block height from the local node
                let (local_client, _) = Client::get_local_node().await?;
                let local_client_res = local_client
                    .view_ext("0x1::block::get_current_block_height", None, None)
                    .await?;

                let block_height_value_local = &local_client_res[0];
                let local_client_block_height =
                    if let Some(block_height_str) = block_height_value_local.as_str() {
                        block_height_str.parse::<u64>()?
                    } else {
                        return Err(anyhow::anyhow!(
                            "Block height from local client is not a string"
                        ));
                    };

                // Get the block height from the working upstream
                app_cfg.refresh_network_profile_and_save(Some(app_cfg.workspace.default_chain_id)).await?;
                let upstream_client_url = app_cfg.pick_url()?;
                let upstream_client = Client::new(upstream_client_url);
                let upstream_client_res = upstream_client
                    .view_ext("0x1::block::get_current_block_height", None, None)
                    .await?;

                let block_height_value_upstream = &upstream_client_res[0];
                let upstream_client_block_height =
                    if let Some(block_height_str) = block_height_value_upstream.as_str() {
                        block_height_str.parse::<u64>()?
                    } else {
                        return Err(anyhow::anyhow!(
                            "Block height from upstream client is not a string"
                        ));
                    };

                // Calculate the sync delay
                let sync_delay = upstream_client_block_height - local_client_block_height;

                Ok(OutputType::Json(serde_json::to_string(&json!({
                    "sync-delay": sync_delay
                }))?))
            }
            QueryType::Txs {
                account,
                start,
                limit,
            } => {
                let txns = client
                    .get_account_transactions(*account, *start, *limit)
                    .await?;
                let transactions = txns.into_inner();
                let json_data = serde_json::to_value(transactions)?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
            QueryType::Events {
                account,
                struct_tag,
                field_name,
                start,
                limit,
            } => {
                let events = client
                    .get_account_events(*account, struct_tag, field_name, *start, *limit)
                    .await?;

                let json_data = serde_json::to_value(events.into_inner())?;
                Ok(OutputType::Json(serde_json::to_string_pretty(&json_data)?))
            }
        }
    }
}
