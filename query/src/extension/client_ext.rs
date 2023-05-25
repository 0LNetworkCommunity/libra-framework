use anyhow::{Context, Result};
use async_trait::async_trait;
use zapatos_sdk::{rest_client::Client, types::account_address::AccountAddress};

#[async_trait]
pub trait ClientExt {
    async fn get_account_resources_ext(&self, account: AccountAddress) -> Result<String>;
}

#[async_trait]
impl ClientExt for Client {
    async fn get_account_resources_ext(&self, account: AccountAddress) -> Result<String> {
        let response = self
            .get_account_resources(account)
            .await
            .context("Failed to get account resources")?;
        Ok(format!("{:#?}", response.inner()))
    }
}
