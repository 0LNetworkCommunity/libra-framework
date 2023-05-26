use crate::util::{format_args, format_type_args, parse_function_id};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use std::time::SystemTime;
use std::{str::FromStr, time::UNIX_EPOCH};
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

#[async_trait]
pub trait ClientExt {
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
