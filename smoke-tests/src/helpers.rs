use std::{
    path::Path,
    time::{Duration, Instant},
};

use anyhow::Context;
use diem_config::config::{NodeConfig, PersistableConfig};
use diem_forge::{DiemPublicInfo, LocalNode, LocalSwarm, NodeExt, Validator};
use diem_genesis::config::HostAndPort;
use diem_sdk::rest_client::Client;

use diem_types::account_address::AccountAddress;
use libra_cached_packages::libra_stdlib;
use libra_config::validator_config;
use libra_types::{
    move_resource::gas_coin::SlowWalletBalance, type_extensions::client_ext::ClientExt,
};
use libra_wallet::core::wallet_library::WalletLibrary;

use smoke_test::test_utils::{MAX_CONNECTIVITY_WAIT_SECS, MAX_HEALTHY_WAIT_SECS};
use std::process::Command;

use crate::libra_smoke::LibraSmoke;

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

pub async fn is_making_progress(client: &Client) -> anyhow::Result<bool> {
    let res = client.get_index().await?;
    let block_height_pre = res.inner().block_height.inner();
    println!("current block height: {}", &block_height_pre);
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);

    while std::time::Instant::now().duration_since(start_time) < timeout {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let progress_check = client.get_index().await?;
        let current_height = progress_check.inner().block_height.inner();

        // make 5 blocks at least
        if *current_height > (*block_height_pre + 5) {
            return Ok(true);
        }
    }
    // no errors but also no progress
    Ok(false)
}

pub fn update_node_config_restart(
    validator: &mut LocalNode,
    config: &mut NodeConfig,
) -> anyhow::Result<()> {
    validator.stop();
    let node_path = validator.config_path();
    config.save_to_path(node_path)?;
    validator.start()?;
    Ok(())
}

pub fn set_sync_only_bool(swarm: &mut LocalSwarm, sync_only: bool) -> anyhow::Result<()> {
    for node in swarm.validators_mut() {
        // Don't load the instantiated config! Get the saved one always!
        let mut node_config = NodeConfig::load_config(node.config_path())?;
        node_config.consensus.sync_only = sync_only;
        update_node_config_restart(node, &mut node_config)?;
    }
    Ok(())
}

pub async fn wait_for_node(
    validator: &mut dyn Validator,
    expected_to_connect: usize,
) -> anyhow::Result<()> {
    let healthy_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_HEALTHY_WAIT_SECS))
        .context("no deadline")?;
    validator
        .wait_until_healthy(healthy_deadline)
        .await
        .unwrap_or_else(|err| {
            let lsof_output = Command::new("lsof").arg("-i").output().unwrap();
            panic!(
                "wait_until_healthy failed. lsof -i: {:?}: {}",
                lsof_output, err
            );
        });
    println!("Validator restart health check passed");

    let connectivity_deadline = Instant::now()
        .checked_add(Duration::from_secs(MAX_CONNECTIVITY_WAIT_SECS))
        .context("can't get new deadline")?;
    validator
        .wait_for_connectivity(expected_to_connect, connectivity_deadline)
        .await?;
    println!("Validator restart connectivity check passed");
    Ok(())
}

pub async fn creates_random_val_account(
    data_path: &Path,
    port: u16,
) -> anyhow::Result<WalletLibrary> {
    let wallet = WalletLibrary::new();
    let mnemonic_string = wallet.mnemonic();

    let my_host = HostAndPort::local(port)?;

    // Initializes the validator configuration.
    validator_config::initialize_validator_files(
        Some(data_path.to_path_buf()),
        Some(&mnemonic_string.clone()[..3]),
        my_host,
        Some(mnemonic_string),
        false,
        Some(diem_types::chain_id::NamedChain::TESTING),
    )
    .await?;

    Ok(wallet)
}

pub async fn make_test_randos(smoke: &LibraSmoke) -> anyhow::Result<()> {
    let env = &smoke.swarm;

    let rando_dir = env.dir().join("rando");

    for (i, local_node) in env.validators().enumerate() {
        creates_random_val_account(&rando_dir.join(i.to_string()), local_node.port()).await?;
    }
    Ok(())
}

// NOTE: Keep this commented code, it's helpful to debug swarm state
// async fn save_debug_dir(from: &Path, to: &str) -> Result<()> {
//     // Get the current directory using CARGO_MANIFEST_DIR
//     let current_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
//         .expect("CARGO_MANIFEST_DIR not set")
//         .into();
//     let to_dir = current_dir.join(to);
//     if to_dir.exists() {
//         tokio::fs::remove_dir_all(&to_dir).await?;
//     }
//     tokio::fs::create_dir_all(&to).await?;
//     fs_extra::dir::copy(
//         from,
//         &to_dir,
//         &fs_extra::dir::CopyOptions::new().content_only(true),
//     )?;
//     Ok(())
// }
