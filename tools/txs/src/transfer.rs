//! form a transfer payload and execute transaction
use super::submit_transaction::Sender;
use anyhow::bail;
use diem_sdk::{
    rest_client::diem_api_types::TransactionOnChainData, types::account_address::AccountAddress,
};
use libra_cached_packages::libra_framework_sdk_builder::EntryFunctionCall::OlAccountTransfer;
use libra_types::move_resource::gas_coin;

impl Sender {
    pub async fn transfer(
        &mut self,
        to: AccountAddress,
        amount: f64,
        estimate: bool,
    ) -> anyhow::Result<Option<TransactionOnChainData>> {
        // must scale the coin from decimal to onchain representation
        let coin_scaled = gas_coin::cast_decimal_to_coin(amount);
        let payload = OlAccountTransfer {
            to,
            amount: coin_scaled,
        }
        .encode();

        if estimate {
            let res = self.estimate(payload).await?;
            println!("{:#?}", &res);

            let success = res[0].info.success;
            println!("will succeed: {success}");
            let gas = res[0].info.gas_used;
            println!("gas used: {gas}");
            Ok(None)
        } else {
            match self.sign_submit_wait(payload).await {
                Ok(tx) => Ok(Some(tx)),
                Err(e) => {
                    bail!(
                        "ERROR: transaction could not complete, message: {}",
                        e.to_string()
                    )
                }
            }
        }
    }
}
