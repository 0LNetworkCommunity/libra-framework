use anyhow::Context;
use diem_forge::DiemPublicInfo;
use diem_sdk::{rest_client::Client, types::LocalAccount};
use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;
use libra_types::{
    move_resource::gas_coin::SlowWalletBalance, type_extensions::client_ext::ClientExt,
};

/// Get the balance of the 0L coin
pub async fn get_libra_balance(
    client: &Client,
    address: AccountAddress,
) -> anyhow::Result<SlowWalletBalance> {
    let res = client
        .view_ext("0x1::ol_account::balance", None, Some(address.to_string()))
        .await?;

    let move_tuple = serde_json::from_value::<Vec<String>>(res)?;

    let b = SlowWalletBalance {
        unlocked: move_tuple[0].parse().context("no value found")?,
        total: move_tuple[1].parse().context("no value found")?,
    };

    Ok(b)
}

pub async fn transfer(
    public_info: &mut DiemPublicInfo<'_>,
    sender_local: &mut LocalAccount,
    addr: AccountAddress,
    amount: u64,
) -> anyhow::Result<()> {
    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::ol_account_transfer(addr, amount));

    let tx = sender_local.sign_with_transaction_builder(payload);

    public_info.client().submit_and_wait(&tx).await?;
    Ok(())
}

pub async fn create_account(
    public_info: &mut DiemPublicInfo<'_>,
    addr: AccountAddress,
) -> anyhow::Result<()> {
    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::ol_account_create_account(addr));

    let mint_txn = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info.client().submit_and_wait(&mint_txn).await?;
    Ok(())
}

pub async fn mint_libra(
    public_info: &mut DiemPublicInfo<'_>,
    addr: AccountAddress,
    amount: u64,
) -> anyhow::Result<()> {
    let payload = public_info
        .transaction_factory()
        .payload(libra_stdlib::libra_coin_mint_to_impl(addr, amount));

    let mint_txn = public_info
        .root_account()
        .sign_with_transaction_builder(payload);

    public_info.client().submit_and_wait(&mint_txn).await?;
    Ok(())
}

pub async fn unlock_libra(
    public_info: &mut DiemPublicInfo<'_>,
    addr: AccountAddress,
    amount: u64,
) -> anyhow::Result<()> {
    // NOTE: assumes the account already has a slow wallet struct
    let unlock_payload =
        public_info
            .transaction_factory()
            .payload(libra_stdlib::slow_wallet_smoke_test_vm_unlock(
                addr, amount, amount,
            ));

    let unlock_txn = public_info
        .root_account()
        .sign_with_transaction_builder(unlock_payload);

    public_info.client().submit_and_wait(&unlock_txn).await?;
    Ok(())
}
