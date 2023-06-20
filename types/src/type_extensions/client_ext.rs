use crate::util::{format_args, format_type_args, parse_function_id};
use crate::type_extensions::cli_config_ext::CliConfigExt;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use std::time::SystemTime;
use std::{str::FromStr, time::UNIX_EPOCH};
use zapatos::common::types::{CliConfig, ConfigSearchMode, DEFAULT_PROFILE};
use zapatos_sdk::{
    move_types::{
        language_storage::{ModuleId, TypeTag},
        parser::{parse_transaction_arguments, parse_type_tags},
        transaction_argument::convert_txn_args,
    },
    rest_client::{
        aptos_api_types::{EntryFunctionId, MoveType, ViewRequest},
        Account, Client,
    },
    transaction_builder::TransactionBuilder,
    types::{
        account_address::AccountAddress,
        chain_id::ChainId,
        transaction::{EntryFunction, SignedTransaction, TransactionArgument, TransactionPayload},
        LocalAccount,
    },
};
use url::Url;
use std::time::Duration;

pub const DEFAULT_TIMEOUT_SECS: u64 = 10;
pub const USER_AGENT: &str = concat!("libra-config/", env!("CARGO_PKG_VERSION"));


#[async_trait]
pub trait ClientExt {
    fn default() -> Result<Client>;

    async fn get_account_balance_libra(&self, account: AccountAddress) -> Result<Vec<serde_json::Value>>;

    async fn get_account_resources_ext(&self, account: AccountAddress) -> Result<String>;

    async fn get_sequence_number(&self, account: AccountAddress) -> Result<u64>;

    async fn generate_transaction(
        &self,
        from_account: &mut LocalAccount,
        function_id: &str,
        ty_args: Option<String>,
        args: Option<String>,
        options: TransactionOptions,
    ) -> Result<SignedTransaction>;

    async fn view_ext(
        &self,
        function_id: &str,
        ty_args: Option<String>,
        args: Option<String>,
    ) -> Result<Vec<serde_json::Value>>;
}

#[async_trait]
impl ClientExt for Client {
      fn default() -> Result<Client> {
        let workspace = crate::global_config_dir().parent().unwrap().to_path_buf();
        let profile =
            CliConfig::load_profile_ext( 
              Some(DEFAULT_PROFILE),
              Some(workspace),
              ConfigSearchMode::CurrentDir
        )?
          .unwrap_or_default();
        let rest_url = profile.rest_url.context("Rest url is not set")?;
        Ok(Client::new_with_timeout_and_user_agent(
            Url::from_str(&rest_url).unwrap(),
            Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            USER_AGENT,
        ))
    }

    async fn get_account_balance_libra(&self, account: AccountAddress) -> Result<Vec<serde_json::Value>> {

      let slow_balance_id = entry_function_id("slow_wallet", "unlocked_amount")?;
      let request = ViewRequest {
          function: slow_balance_id,
          type_arguments: vec![],
          arguments: vec![account.to_string().into()],
      };
      
      let res = self.view(&request, None).await?.into_inner();

      dbg!(&res);

      Ok(res)

    }


    async fn get_account_resources_ext(&self, account: AccountAddress) -> Result<String> {
        let response = self
            .get_account_resources(account)
            .await
            .context("Failed to get account resources")?;
        Ok(format!("{:#?}", response.inner()))
    }

    async fn get_sequence_number(&self, account: AccountAddress) -> Result<u64> {
        let response = self
            .get_account_resource(account, "0x1::account::Account")
            .await
            .context("Failed to get account resource")?;
        if let Some(res) = response.inner() {
            Ok(serde_json::from_value::<Account>(res.data.to_owned())?.sequence_number)
        } else {
            Err(anyhow!("No data returned for the sequence number"))
        }
    }

    async fn generate_transaction(
        &self,
        from_account: &mut LocalAccount,
        function_id: &str,
        ty_args: Option<String>,
        args: Option<String>,
        options: TransactionOptions,
    ) -> Result<SignedTransaction> {
        let chain_id = self.get_index().await?.inner().chain_id;
        let (module_address, module_name, function_name) = parse_function_id(function_id)?;
        let module = ModuleId::new(module_address, module_name);
        let ty_args: Vec<TypeTag> = if let Some(ty_args) = ty_args {
            parse_type_tags(&ty_args)
                .context(format!("Unable to parse the type argument(s): {ty_args}"))?
        } else {
            vec![]
        };
        let args: Vec<TransactionArgument> = if let Some(args) = args {
            parse_transaction_arguments(&args)
                .context(format!("Unable to parse argument(s): {args}"))?
        } else {
            vec![]
        };

        println!("{}", format_type_args(&ty_args));
        println!("{}", format_args(&args));

        let expiration_timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + options.timeout_secs;

        let transaction_builder = TransactionBuilder::new(
            TransactionPayload::EntryFunction(EntryFunction::new(
                module,
                function_name,
                ty_args,
                convert_txn_args(&args),
            )),
            expiration_timestamp_secs,
            ChainId::new(chain_id),
        )
        .max_gas_amount(options.max_gas_amount)
        .gas_unit_price(options.gas_unit_price);

        Ok(from_account.sign_with_transaction_builder(transaction_builder))
    }

    // async fn view_bcs(
    //     &self,
    //     request: &ViewRequest,
    //     version: Option<u64>,
    // ) -> Result<bytes::Bytes> {
    //     let request = serde_json::to_string(request)?;
    //     let mut url = self.build_path("view")?;
    //     if let Some(version) = version {
    //         url.set_query(Some(format!("ledger_version={}", version).as_str()));
    //     }

    //     let response = self
    //         .inner
    //         .post(url)
    //         .header(CONTENT_TYPE, JSON)
    //         .body(request)
    //         .send()
    //         .await?;

    //     Ok(self.check_and_parse_bcs_response(response).await?.inner())
    // }

    async fn view_ext(
        &self,
        function_id: &str,
        ty_args: Option<String>,
        args: Option<String>,
    ) -> Result<Vec<serde_json::Value>> {
        let entry_fuction_id = EntryFunctionId::from_str(function_id)
            .context(format!("Invalid function id: {function_id}"))?;
        let ty_args: Vec<MoveType> = if let Some(ty_args) = ty_args {
            parse_type_tags(&ty_args)
                .context(format!("Unable to parse the type argument(s): {ty_args}"))?
                .iter()
                .map(|t| t.into())
                .collect()
        } else {
            vec![]
        };
        let args: Vec<serde_json::Value> = if let Some(args) = args {
            let mut output = vec![];
            for arg in args.split(',') {
                let arg = serde_json::Value::try_from(arg.trim())
                    .context(format!("Failed to parse argument: {arg}"))?;
                output.push(arg);
            }
            output
        } else {
            vec![]
        };

        println!("{}", format_type_args(&ty_args));
        println!("{}", format_args(&args));

        let request = ViewRequest {
            function: entry_fuction_id,
            type_arguments: ty_args,
            arguments: args,
        };

        self.view(&request, None)
            .await
            .context("Failed to execute View request")
            .map(|res| res.inner().to_owned())
    }
}

pub struct TransactionOptions {
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub timeout_secs: u64,
}


pub fn entry_function_id(module_name: &str, function_name: &str) -> Result<EntryFunctionId> {
  let s = format!("0x1::{}::{}", module_name, function_name);
  EntryFunctionId::from_str(&s)
      .context(format!("Invalid function id: {s}"))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Person {
    x: f64,
    y: f64,
}


#[test]
fn serde_test() {



    let s = r#"{"x": 1.0, "y": 2.0}"#;
    let mut value: serde_json::Value = serde_json::from_str(s).unwrap();
    // value.
    dbg!(&value);

    let p: Person = serde_json::from_value(value).unwrap();
    dbg!(&p);

}