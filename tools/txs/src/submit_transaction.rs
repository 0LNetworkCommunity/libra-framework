use anyhow::bail;
use zapatos::{
    account::key_rotation::lookup_address,
    common::types::{CliConfig, ConfigSearchMode},
};
use zapatos_sdk::{
    rest_client::{aptos_api_types::TransactionOnChainData, Client},
    transaction_builder::TransactionBuilder,
    types::{
        chain_id::ChainId,
        transaction::{ExecutionStatus, SignedTransaction, TransactionPayload},
        AccountKey, LocalAccount,
    },
};

use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

use libra_types::{
    exports::Ed25519PrivateKey,
    legacy_types::app_cfg::AppCfg,
    type_extensions::{
        cli_config_ext::CliConfigExt,
        client_ext::{ClientExt, DEFAULT_TIMEOUT_SECS},
    },
};

// #[derive(Debug)]
// /// a transaction error type specific to ol txs
// pub struct TxError {
//     /// the actual error type
//     pub err: Option<Error>,
//     /// transaction view if the transaction got that far
//     pub tx_view: Option<TransactionView>,
//     /// Move module or script where error occurred
//     pub location: Option<String>,
//     /// Move abort code used in error
//     pub abort_code: Option<u64>,
// }

// impl From<anyhow::Error> for TxError {
//     fn from(e: anyhow::Error) -> Self {
//         TxError {
//             err: Some(e),
//             tx_view: None,
//             location: None,
//             abort_code: None,
//         }
//     }
// }

// pub async fn submit(signed_trans: &SignedTransaction) -> anyhow::Result<String> {
//     let client = Client::default()?;
//     let pending_trans = client.submit(signed_trans).await?.into_inner();
//     let res = client.wait_for_transaction(&pending_trans).await?;

//     match res.inner().success() {
//         true => {
//           // TODO: use logger crate
//           println!("transaction success!")
//           return Ok(res.inner().vm_status())
//         },
//         false => {
//           println!("transaction not successful, status: {:?}", &res.inner().vm_status());
//           return bail!("transaction not successful, status: {:?}", &res.inner().vm_status());
//         },
//     }
//     Ok(())
// }

/// Struct to organize all the TXS sending, so we're not creating new Client on every TX, if there are multiple.
pub struct Sender {
    client: Client,
    local_account: LocalAccount,
    chain_id: ChainId,
    response: Option<TransactionOnChainData>,
}

impl Sender {
    pub async fn new(
        account_key: AccountKey,
        chain_id: ChainId,
        client_opt: Option<Client>,
    ) -> anyhow::Result<Self> {
        let client = client_opt.unwrap_or(Client::default()?);

        let address = lookup_address(
            &client,
            account_key.authentication_key().derived_address(),
            false,
        )
        .await?;

        let seq = client.get_sequence_number(address).await?;
        let local_account = LocalAccount::new(address, account_key, seq);

        Ok(Self {
            client,
            local_account,
            chain_id,
            response: None,
        })
    }
    pub async fn from_app_cfg(
        app_cfg: &AppCfg,
        pri_key: Option<Ed25519PrivateKey>,
    ) -> anyhow::Result<Self> {
        let address = app_cfg.profile.account;

        let key = match pri_key {
            Some(p) => p,
            None => {
                match app_cfg.profile.test_private_key.clone() {
                    Some(k) => k,
                    None => {
                      let leg_keys =  libra_wallet::legacy::get_keys_from_prompt()?;
                      leg_keys.child_0_owner.pri_key
                    },
                }
            }
        };

        let temp_seq_num = 0;
        let mut local_account = LocalAccount::new(address, key, temp_seq_num);

        let nodes = &app_cfg.profile.upstream_nodes;
        // Todo: find one out of the list which can produce metadata, not the first one.
        let url: Url = match nodes.iter().next() {
            Some(u) => u.to_owned(),
            None => bail!("could not find rest_url in profile"),
        };

        // check if we can connect to this client, or exit
        let client = Client::new(url.clone());

        // let mut chain_id = ChainId::testnet();
        let seq_num = local_account.sequence_number_mut();

        let chain_id = match client.get_index().await {
            Ok(metadata) => {
                *seq_num = client.get_sequence_number(address).await?;
                ChainId::new(metadata.into_inner().chain_id)
            }
            Err(_) => bail!("cannot connect to client at {:?}", &url),
        };

        let s = Sender {
            client,
            local_account,
            chain_id,
            response: None,
        };

        return Ok(s);
    }

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
                    let leg_keys = libra_wallet::legacy::get_keys_from_prompt()?;
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

    pub async fn sign_submit_wait(
        &mut self,
        payload: TransactionPayload,
    ) -> anyhow::Result<TransactionOnChainData> {
        let signed = self.sign_payload(payload);
        let r = self.submit(&signed).await?;
        self.response = Some(r.clone());
        Ok(r)
    }

    pub fn sign_payload(&mut self, payload: TransactionPayload) -> SignedTransaction {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let time = t + DEFAULT_TIMEOUT_SECS * 10;
        let tb = TransactionBuilder::new(payload, time, self.chain_id);
        self.local_account.sign_with_transaction_builder(tb)
    }

    /// submit to API and wait for the transaction on chain data
    pub async fn submit(
        &mut self,
        signed_trans: &SignedTransaction,
    ) -> anyhow::Result<TransactionOnChainData> {
        let pending_trans = self.client.submit(signed_trans).await?.into_inner();

        let res = self
            .client
            .wait_for_transaction_bcs(&pending_trans)
            .await?
            .into_inner();

        Ok(res)
    }
    pub fn eval_response(&self) -> anyhow::Result<ExecutionStatus, ExecutionStatus> {
        if self.response.is_none() {
            return Err(ExecutionStatus::MiscellaneousError(None))
        };
        let status = self.response.as_ref().unwrap().info.status();
        match status.is_success() {
            true => {
                println!("transaction success!");
                Ok(status.to_owned())
            }
            false => {
                println!("transaction not successful, status: {:?}", &status);
                Err(status.to_owned())
            }
        }
    }
}
