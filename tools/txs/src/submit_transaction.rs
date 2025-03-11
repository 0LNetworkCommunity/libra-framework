//! Module for managing transaction sending and management with Diem blockchain.

use crate::txs_cli::to_legacy_address;
use anyhow::{bail, Context};
use diem::common::types::{CliConfig, ConfigSearchMode};
use diem_logger::prelude::*;
use diem_sdk::{
    crypto::{HashValue, PrivateKey},
    rest_client::{
        diem_api_types::{TransactionOnChainData, UserTransaction},
        Client,
    },
    transaction_builder::TransactionBuilder,
    types::{
        chain_id::ChainId,
        transaction::{ExecutionStatus, SignedTransaction, TransactionPayload},
        AccountKey, LocalAccount,
    },
};
use libra_types::{
    core_types::app_cfg::{AppCfg, TxCost},
    exports::{AuthenticationKey, Ed25519PrivateKey},
    ol_progress::OLProgress,
    type_extensions::{
        cli_config_ext::CliConfigExt,
        client_ext::{ClientExt, DEFAULT_TIMEOUT_SECS},
    },
};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

/// Struct to organize all the TXS sending, so we're not creating new Client on every TX, if there are multiple.
pub struct Sender {
    pub local_account: LocalAccount,
    pub tx_cost: TxCost,
    client: Client,
    chain_id: ChainId,
    pub response: Option<TransactionOnChainData>,
}

impl Sender {
    pub async fn new(
        account_key: AccountKey,
        chain_id: ChainId,
        client_opt: Option<Client>,
        use_legacy_address: bool,
    ) -> anyhow::Result<Self> {
        let client = match client_opt {
            Some(c) => c,
            None => Client::default().await?,
        };

        // Lookup the originating address and handle legacy address conversion if necessary
        let address = client
            .lookup_originating_address(account_key.authentication_key())
            .await
            .map(|address| {
                if !use_legacy_address {
                    return Ok(address);
                }

                to_legacy_address(&address)
            })??;
        info!("using address {}", &address);

        // Fetch sequence number for the account
        let seq = client.get_sequence_number(address).await?;
        let local_account = LocalAccount::new(address, account_key, seq);

        Ok(Self {
            client,
            tx_cost: TxCost::default_baseline_cost(),
            local_account,
            chain_id,
            response: None,
        })
    }

    /// Sets the transaction cost for the `Sender`.
    pub fn set_tx_cost(&mut self, cost: &TxCost) {
        cost.clone_into(&mut self.tx_cost);
    }

    /// load from local app configs
    pub async fn from_app_cfg(app_cfg: &AppCfg, profile: Option<String>) -> anyhow::Result<Self> {
        let profile = app_cfg.get_profile(profile)?;

        let key = match profile.borrow_private_key() {
            Ok(k) => k.to_owned(),
            _ => {
                let leg_keys = libra_wallet::account_keys::get_keys_from_prompt()?;
                leg_keys.child_0_owner.pri_key
            }
        };

        let temp_seq_num = 0;

        // Create authentication key and initialize client with URL
        let auth_key = AuthenticationKey::ed25519(&key.public_key());
        let url = &app_cfg.pick_url(None)?;
        let client = Client::new(url.clone());
        let address = client
            .lookup_originating_address(auth_key)
            .await
            .unwrap_or(profile.account);

        let mut local_account = LocalAccount::new(address, key, temp_seq_num);
        let seq_num = local_account.sequence_number_mut();

        // check if we can connect to this client, or exit
        let chain_id = match client.get_index().await {
            Ok(metadata) => {
                // update sequence number
                *seq_num = client
                    .get_sequence_number(address)
                    .await
                    .context("failed to get sequence number")?;
                ChainId::new(metadata.into_inner().chain_id)
            }
            Err(_) => bail!("cannot connect to client at {:?}", &url),
        };

        let s = Sender {
            client,
            tx_cost: app_cfg.tx_configs.get_cost(None),
            local_account,
            chain_id,
            response: None,
        };

        Ok(s)
    }

    // TODO: is this deprecated
    pub async fn from_vendor_profile(
        profile_name: Option<&str>,
        workspace: Option<PathBuf>,
        pri_key: Option<Ed25519PrivateKey>,
    ) -> anyhow::Result<Self> {
        let cfg = CliConfig::load_profile_ext(None, None, ConfigSearchMode::CurrentDir)?;
        if let Some(c) = cfg {
            let address = match c.account {
                Some(acc) => acc,
                None => bail!("no profile found"),
            };

            let key = match pri_key {
                Some(p) => p,
                None => {
                    let leg_keys = libra_wallet::account_keys::get_keys_from_prompt()?;
                    leg_keys.child_0_owner.pri_key
                }
            };

            let temp_seq_num = 0;
            let mut local_account = LocalAccount::new(address, key, temp_seq_num);

            let url: Url = match c.rest_url {
                Some(url_str) => url_str.parse()?,
                None => bail!("could not find rest_url in profile"),
            };

            // check if we can connect to this client, or exit
            let client = Client::new(url.clone());

            let seq_num = match client.get_index().await {
                Ok(_) => client.get_sequence_number(address).await?,
                Err(_) => bail!("cannot connect to client at {:?}", &url),
            };

            let s = local_account.sequence_number_mut();
            *s = seq_num;
            // update the sequence number of account.
            let chain_id = match c.network {
                Some(net) => ChainId::new(net as u8),
                None => bail!("cannot get which network id to connect to"),
            };

            let s = Sender {
                client,
                tx_cost: TxCost::default_baseline_cost(),
                local_account,
                chain_id,
                response: None,
            };
            return Ok(s);
        }
        bail!(
            "could not read profile: {:?} at {:?}",
            profile_name,
            workspace
        );
    }

    /// Signs and submits a transaction payload, waiting for the transaction on-chain data.
    pub async fn sign_submit_wait(
        &mut self,
        payload: TransactionPayload,
    ) -> anyhow::Result<TransactionOnChainData> {
        if let TransactionPayload::Script(s) = &payload {
            let hash = HashValue::sha3_256_of(s.code());
            info!("script code hash: {}", &hash.to_hex_literal());
        }

        let signed = self.sign_payload(payload);
        let spin = OLProgress::spin_steady(500, "awaiting transaction response".to_string());
        println!("sending transaction...");
        let r = self.submit(&signed).await?;
        println!("transaction sent");
        self.response = Some(r.clone());
        spin.finish_and_clear();
        // debug!("{:?}", &r);
        OLProgress::complete("transaction success");
        Ok(r)
    }

    /// sync helper for sending tx
    pub fn sync_sign_submit_wait(
        &mut self,
        payload: TransactionPayload,
    ) -> anyhow::Result<TransactionOnChainData> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Call the asynchronous connect method using the runtime.
        rt.block_on(self.sign_submit_wait(payload))
    }

    /// Signs a transaction payload.
    pub fn sign_payload(&mut self, payload: TransactionPayload) -> SignedTransaction {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let time = t + (DEFAULT_TIMEOUT_SECS * 10);

        let tb = TransactionBuilder::new(payload, time, self.chain_id)
            .gas_unit_price(self.tx_cost.coin_price_per_unit)
            .max_gas_amount(self.tx_cost.max_gas_unit_for_tx);

        self.local_account.sign_with_transaction_builder(tb)
    }

    /// submit to API and wait for the transaction on chain data
    pub async fn submit(
        &mut self,
        signed_trans: &SignedTransaction,
    ) -> anyhow::Result<TransactionOnChainData> {
        debug!("signed tx payload: {:?}", &signed_trans.payload());
        let pending_trans = self.client.submit(signed_trans).await?.into_inner();

        info!("pending tx hash: {}", &pending_trans.hash.to_string());

        let res = self
            .client
            .wait_for_transaction_bcs(&pending_trans)
            .await?
            .into_inner();

        Ok(res)
    }

    /// Evaluates the response of the last submitted transaction.
    pub fn eval_response(&self) -> anyhow::Result<ExecutionStatus, ExecutionStatus> {
        if self.response.is_none() {
            return Err(ExecutionStatus::MiscellaneousError(None));
        };
        let status = self.response.as_ref().unwrap().info.status();
        match status.is_success() {
            true => Ok(status.to_owned()),
            false => {
                println!("transaction not successful, status: {:?}", &status);
                Err(status.to_owned())
            }
        }
    }

    /// estimate the transaction gas cost.
    pub async fn estimate(
        &mut self,
        payload: TransactionPayload,
    ) -> anyhow::Result<Vec<UserTransaction>> {
        let signed = self.sign_payload(payload);

        let res = self
            .client
            .simulate_with_gas_estimation(&signed, true, true)
            .await?
            .into_inner();
        Ok(res)
    }

    /// get the transactions hash, for use with governance scripts.
    pub fn tx_hash(&self) -> Option<HashValue> {
        if let Some(r) = &self.response {
            return Some(r.info.transaction_hash());
        };
        None
    }

    /// Returns a reference to the underlying client.
    pub fn client(&self) -> &Client {
        &self.client
    }
}
