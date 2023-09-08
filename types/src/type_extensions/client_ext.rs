use crate::exports::AuthenticationKey;
use crate::legacy_types::app_cfg::AppCfg;
use crate::type_extensions::cli_config_ext::CliConfigExt;
use crate::util::parse_function_id;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{self, Value};
// use std::time::Duration;
use diem::common::types::{CliConfig, ConfigSearchMode, DEFAULT_PROFILE};
use diem_sdk::{
    move_types::{
        language_storage::{ModuleId, TypeTag},
        move_resource::MoveStructType,
        parser::{parse_transaction_arguments, parse_type_tags},
        transaction_argument::convert_txn_args,
    },
    rest_client::{
        diem_api_types::{EntryFunctionId, MoveType, ViewRequest},
        Account, Client,
    },
    transaction_builder::TransactionBuilder,
    types::{
        account_address::AccountAddress,
        chain_id::{ChainId, NamedChain},
        transaction::{EntryFunction, SignedTransaction, TransactionArgument, TransactionPayload},
        LocalAccount,
    },
};
use std::time::SystemTime;
use std::{str::FromStr, time::UNIX_EPOCH};
use url::Url;

pub const DEFAULT_TIMEOUT_SECS: u64 = 10;
pub const USER_AGENT: &str = concat!("libra-config/", env!("CARGO_PKG_VERSION"));
pub const LOCAL_NODE_URL: &str = "http://localhost:8080";

#[async_trait]
pub trait ClientExt {
    async fn default() -> anyhow::Result<Client>;

    async fn get_local_node() -> anyhow::Result<(Client, ChainId)>;

    async fn from_libra_config(
        app_cfg: &AppCfg,
        chain_id_opt: Option<NamedChain>,
    ) -> anyhow::Result<(Client, ChainId)>;

    fn from_vendor_config() -> anyhow::Result<Client>;

    async fn lookup_originating_address(
        &self,
        authentication_key: AuthenticationKey,
    ) -> anyhow::Result<AccountAddress>;

    async fn get_move_resource<T: MoveStructType + DeserializeOwned>(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<T>;

    async fn get_account_resources_ext(&self, account: AccountAddress) -> anyhow::Result<String>;

    async fn get_sequence_number(&self, account: AccountAddress) -> anyhow::Result<u64>;

    async fn generate_transaction(
        &self,
        from_account: &mut LocalAccount,
        function_id: &str,
        ty_args: Option<String>,
        args: Option<String>,
        options: TransactionOptions,
    ) -> anyhow::Result<SignedTransaction>;

    async fn view_ext(
        &self,
        function_id: &str,
        ty_args: Option<String>,
        args: Option<String>,
    ) -> anyhow::Result<Value>;
}

#[async_trait]
impl ClientExt for Client {
    /// assumes the location of the config files, and gets a node from list in config
    async fn default() -> anyhow::Result<Client> {
        let app_cfg = AppCfg::load(None)?;
        let (client, _) = Self::from_libra_config(&app_cfg, None).await?;
        Ok(client)
    }

    /// Gets the local node
    async fn get_local_node() -> anyhow::Result<(Client, ChainId)> {
        let local_url = Url::parse(LOCAL_NODE_URL)?;
        let client = Client::new(local_url);
        match client.get_index().await {
            Ok(res) => Ok((client, ChainId::new(res.inner().chain_id))),
            Err(e) => Err(anyhow::anyhow!(
                "Failed to connect to the local node: {}",
                e
            )),
        }
    }

    /// Finds a good working upstream based on the list in a config file
    async fn from_libra_config(
        app_cfg: &AppCfg,
        _chain_id_opt: Option<NamedChain>,
    ) -> anyhow::Result<(Client, ChainId)> {
        // check if we can connect to this client, or exit
        let url = &app_cfg.pick_url()?;
        let client = Client::new(url.to_owned());
        let res = client.get_index().await?;

        Ok((client, ChainId::new(res.inner().chain_id)))
    }

    fn from_vendor_config() -> anyhow::Result<Client> {
        let workspace = crate::global_config_dir().parent().unwrap().to_path_buf();
        let profile = CliConfig::load_profile_ext(
            Some(DEFAULT_PROFILE),
            Some(workspace),
            ConfigSearchMode::CurrentDir,
        )?
        .unwrap_or_default();
        let rest_url = profile.rest_url.context("Rest url is not set")?;
        Ok(Client::new(Url::from_str(&rest_url).unwrap()))
    }

    /// Addresses will diverge from the keypair which originally created the address.
    /// The Address and AuthenticationKey key are the same bytes: a sha3 hash
    /// of the public key. If you rotate the keypair for that address, the implied (derived) public key, and thus authentication key will not be the same as the
    ///  Origial/originating address. For this reason, we need to look up the original address
    /// Addresses are stored in the OriginatingAddress table, which is a table
    /// that maps a derived address to the original address. This function
    /// looks up the original address for a given derived address.
    async fn lookup_originating_address(
        &self,
        authentication_key: AuthenticationKey,
    ) -> anyhow::Result<AccountAddress> {
        // the move View will return the same address_key if it has an unmodified Authkey (never been rotated)
        // let bytes = authentication_key.to_vec();
        // let cast_address = AccountAddress::from_bytes(bytes.as_slice())?;

        let function_id = entry_function_id("account", "get_originating_address")?;
        let request = ViewRequest {
            function: function_id,
            type_arguments: vec![],
            arguments: vec![authentication_key.to_string().into()],
        };

        let res = self.view(&request, None).await?.into_inner();
        let addr = serde_json::from_value(res[0].clone())?;
        Ok(addr)
    }

    async fn get_move_resource<T: MoveStructType + DeserializeOwned>(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<T> {
        let resource_type = format!("0x1::{}::{}", T::MODULE_NAME, T::STRUCT_NAME);
        let res = self
            .get_account_resource_bcs::<T>(address, &resource_type)
            .await?
            .into_inner();

        Ok(res)
    }

    async fn get_account_resources_ext(&self, account: AccountAddress) -> anyhow::Result<String> {
        let response = self
            .get_account_resources(account)
            .await
            .context("Failed to get account resources")?;
        Ok(format!("{:#?}", response.inner()))
    }

    async fn get_sequence_number(&self, account: AccountAddress) -> anyhow::Result<u64> {
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
    ) -> anyhow::Result<SignedTransaction> {
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

        // println!("{}", format_type_args(&ty_args));
        // println!("{}", format_args(&args));

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
    // ) -> anyhow::Result<bytes::Bytes> {
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
    ) -> anyhow::Result<Value> {
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

        // println!("{}", format_type_args(&ty_args));
        // println!("{}", format_args(&args));

        let request = ViewRequest {
            function: entry_fuction_id,
            type_arguments: ty_args,
            arguments: args,
        };

        let array = self
            .view(&request, None)
            .await
            .context("Failed to execute View request")
            .map(|res| res.inner().to_owned())?;
        Ok(Value::Array(array))
    }
}

pub struct TransactionOptions {
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub timeout_secs: u64,
}

pub fn entry_function_id(
    module_name: &str,
    function_name: &str,
) -> anyhow::Result<EntryFunctionId> {
    let s = format!("0x1::{}::{}", module_name, function_name);
    EntryFunctionId::from_str(&s).context(format!("Invalid function id: {s}"))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Person {
    x: f64,
    y: f64,
}

#[test]
fn serde_test() {
    let s = r#"{"x": 1.0, "y": 2.0}"#;
    let value: serde_json::Value = serde_json::from_str(s).unwrap();
    // value.
    dbg!(&value);

    let p: Person = serde_json::from_value(value).unwrap();
    dbg!(&p);
}
