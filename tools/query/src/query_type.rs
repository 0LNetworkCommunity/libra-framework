use crate::{
    account_queries::{ get_account_balance_libra, get_tower_state},
    query_view::{ fetch_and_display },
    utils::{ colorize_and_print, print_colored_kv },

};
use anyhow::{bail, Result, anyhow};
use indoc::indoc;
use libra_types::exports::AuthenticationKey;
use libra_types::type_extensions::client_ext::ClientExt;
use serde_json::Value;
use zapatos_sdk::{rest_client::Client, types::account_address::AccountAddress};

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
            let json_data = get_account_balance_libra(&client, *account).await?;
            let json_str = serde_json::to_string_pretty(&json_data)?;
            match colorize_and_print(&json_str) {
                Ok(_) => (), 
                Err(err) => {
                    eprintln!("Error colorizing JSON: {}", err);
                    println!("{}", json_str); 
                }
            };
            Ok(Value::String("Success".to_string()))
        },
        QueryType::Tower { account } => {
            let tower_state = get_tower_state(&client, *account).await?;
            let json_data = serde_json::to_value(tower_state)?;
            let json_str = serde_json::to_string_pretty(&json_data)?;
            match colorize_and_print(&json_str) {
                Ok(_) => (), 
                Err(err) => {
                    eprintln!("Error colorizing JSON: {}", err);
                    println!("{}", json_str); 
                }
            };
            Ok(Value::String("Success".to_string()))
        },
        QueryType::View { function_id, type_args, args } => {
            let json = fetch_and_display(&client, function_id, type_args.to_owned(), args.to_owned()).await?;
            Ok(Value::String("Success".to_string()))
        },
        QueryType::Epoch => {
            let json = fetch_and_display(&client, "0x1::reconfiguration::get_current_epoch", None, None).await?;
            Ok(Value::String("Success".to_string()))
        },
        QueryType::BlockHeight => {
            let json = fetch_and_display(&client, "0x1::block::get_current_block_height", None, None).await?;
            Ok(Value::String("Success".to_string()))
        },
        QueryType::Resources { account } => {
            let res = &client.get_account_resources(*account)
                .await?
                .into_inner()
                .into_iter()
                .map(|resource| {
                    let mut map = serde_json::Map::new();
                    map.insert(resource.resource_type.to_string(), resource.data);
                    serde_json::Value::Object(map)
                })
                .collect::<Vec<serde_json::Value>>();

            let json_str = serde_json::to_string_pretty(&res).expect("Failed to serialize to JSON");
            colorize_and_print(&json_str)?;
            Ok(Value::String("Success".to_string()))
        },
        QueryType::MoveValue { account, module_name, struct_name, key_name } => {
     
            let res = &client.get_account_resources(*account)
            .await?
            .into_inner()
            .into_iter()
            .map(|resource| {
                let mut map = serde_json::Map::new();
                map.insert(resource.resource_type.to_string(), resource.data);
                serde_json::Value::Object(map)
            })
            .collect::<Vec<serde_json::Value>>();
        
            
            let module_search_pattern = format!("::{}::", module_name);

            let maybe_module_struct = res.iter().find(|value| {
                if let Some(map) = value.as_object() {
                    map.keys().any(|k| k.contains(&module_search_pattern) && k.ends_with(struct_name))
                } else {
                    false
                }
            });
            
            if let Some(module_struct) = maybe_module_struct {
                if let Some(struct_data) = module_struct.as_object().and_then(|map| map.values().next()) {
                    if let Some(key_value) = struct_data.get(key_name) {
                        let account_str = module_struct.as_object().map_or("".to_string(), |map| map.keys().next().cloned().unwrap_or_else(|| "".to_string()));
                        let account_parts: Vec<&str> = account_str.split("::").collect();
                        let account = account_parts.get(0).unwrap_or(&"Unknown");
                        
                        print_colored_kv("account", &account);
                        print_colored_kv("module_name", &module_name);
                        print_colored_kv("module_struct", &struct_name);
                        

                        let parsed_value = key_value.as_str().and_then(|s| s.parse::<i32>().ok()).unwrap_or_default();
                        print_colored_kv(key_name, &parsed_value.to_string());

                        Ok(Value::String("Success".to_string()))
                    } else {
                        return Err(anyhow!("Key '{}' not found in struct '{}'", key_name, struct_name));
                    }
                } else {
                    return Err(anyhow!("Struct '{}' not found in module '{}'", struct_name, module_name));
                }
            } else {
                return Err(anyhow!("Module '{}' not found", module_name));
            }
            
   
        }
        
        
        _ => { bail!("Not implemented for type: {:?}", self) }
       

    }
    }
}
