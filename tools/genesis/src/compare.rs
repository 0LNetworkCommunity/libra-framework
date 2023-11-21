//! functions for comparing LegacyRecovery data to a genesis blob
//!
//! every day is like sunday
//! -- morrissey via github copilot
use crate::genesis_functions::util_simulate_new_val_balance;
use crate::genesis_reader::total_supply;
use crate::supply::Supply;
use crate::{genesis_reader, parse_json};

use anyhow::{self, Context};
use diem_state_view::account_with_state_view::AsAccountWithStateView;
use diem_storage_interface::state_view::LatestDbStateCheckpointView;
use diem_storage_interface::DbReader;
use diem_types::account_view::AccountView;
use diem_types::transaction::Transaction;
use indicatif::{ProgressBar, ProgressIterator};
use libra_types::exports::AccountAddress;
use libra_types::legacy_types::legacy_address::LegacyAddress;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
use libra_types::move_resource::gas_coin::{GasCoinStoreResource, SlowWalletBalance};
use libra_types::ol_progress::OLProgress;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
/// struct for holding the results of a comparison
pub struct CompareError {
    /// index of LegacyRecover
    pub index: u64,
    /// user account
    pub account: Option<LegacyAddress>,
    /// value expected
    pub expected: u64,
    /// value on chain after migration
    pub migrated: u64,
    /// error message
    pub message: String,
}

/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_recovery_vec_to_genesis_tx(
    recovery: &mut [LegacyRecovery],
    db_reader: &Arc<dyn DbReader>,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    let mut err_list: Vec<CompareError> = vec![];

    recovery
        .iter_mut()
        .progress_with_style(OLProgress::bar())
        .with_message("auditing migration")
        .enumerate()
        .for_each(|(i, old)| {
            if old.account.is_none() {
                err_list.push(CompareError {
                    index: i as u64,
                    account: None,
                    expected: 0,
                    migrated: 0,
                    message: "account is None".to_string(),
                }); // instead of balance, if there is an account that is None, we insert the index of the recovery file
                return;
            };

            if old.account.unwrap() == LegacyAddress::ZERO {
                return;
            };

            let convert_address =
                AccountAddress::from_hex_literal(&old.account.as_ref().unwrap().to_hex_literal())
                    .expect("could not convert address types");

            // // scale all coin values per record consistently
            // util_scale_all_coins(old, supply).expect("could not scale coins");
            util_simulate_new_val_balance(old, supply)
                .expect("could not simulate infra escrow validators");

            // Ok now let's compare to what's on chain
            let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
            let account_state_view = db_state_view.as_account_with_state_view(&convert_address);

            let on_chain_balance = account_state_view
                .get_move_resource::<GasCoinStoreResource>()
                .expect("should have move resource")
                .expect("should have a GasCoinStoreResource for balance");

            // CHECK: we should have scaled the balance correctly, including
            // adjusting for validators
            let old_balance = old.balance.as_ref().expect("should have a balance struct");
            if on_chain_balance.coin() != old_balance.coin {
                err_list.push(CompareError {
                    index: i as u64,
                    account: old.account,
                    expected: old_balance.coin,
                    migrated: on_chain_balance.coin(),
                    message: "unexpected balance".to_string(),
                });
            }

            // Check Slow Wallet Balance was migrated as expected
            if let Some(old_slow) = &old.slow_wallet {
                let new_slow = account_state_view
                    .get_move_resource::<SlowWalletBalance>()
                    .expect("should have a slow wallet struct")
                    .unwrap();

                if new_slow.unlocked != old_slow.unlocked {
                    err_list.push(CompareError {
                        index: i as u64,
                        account: old.account,
                        expected: old_slow.unlocked,
                        migrated: new_slow.unlocked,
                        // bal_diff: on_chain_slow_wallet.unlocked as i64 - expected_unlocked as i64,
                        message: "unexpected slow wallet unlocked".to_string(),
                    });
                }
                // CHECK: the unlocked amount should never be greater than balance
                if new_slow.unlocked > on_chain_balance.coin() {
                    err_list.push(CompareError {
                        index: i as u64,
                        account: old.account,
                        expected: new_slow.unlocked,
                        migrated: on_chain_balance.coin(),
                        message: "unlocked greater than balance".to_string(),
                    });
                }
            }
        });

    Ok(err_list)
}

#[derive(Serialize, Deserialize)]
struct JsonDump {
    account: AccountAddress,
    balance: Option<GasCoinStoreResource>,
    slow: Option<SlowWalletBalance>,
}
/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn export_account_balances(
    recovery: &[LegacyRecovery],
    db_reader: &Arc<dyn DbReader>,
    output: &Path,
) -> anyhow::Result<()> {
    let mut list: Vec<JsonDump> = vec![];

    recovery
        .iter()
        .progress_with_style(OLProgress::bar())
        .with_message("auditing migration")
        .for_each(|old| {
            if old.account.is_none() {
                return;
            };

            let account =
                AccountAddress::from_hex_literal(&old.account.as_ref().unwrap().to_hex_literal())
                    .expect("could not convert address types");

            // Ok now let's compare to what's on chain
            let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
            let account_state_view = db_state_view.as_account_with_state_view(&account);

            let slow = account_state_view
                .get_move_resource::<SlowWalletBalance>()
                .expect("should have a slow wallet struct");

            let balance = account_state_view
                .get_move_resource::<GasCoinStoreResource>()
                .expect("should have move resource");

            list.push(JsonDump {
                account,
                balance,
                slow,
            });
        });

    std::fs::write(
        output.join("genesis_balances.json"),
        serde_json::to_string_pretty(&list).unwrap(),
    )
    .unwrap();
    Ok(())
}

/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_json_to_genesis_blob(
    json_path: PathBuf,
    genesis_path: PathBuf,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    let mut recovery = parse_json::recovery_file_parse(json_path)?;

    let gen_tx = genesis_reader::read_blob_to_tx(genesis_path)?;
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx)?;
    compare_recovery_vec_to_genesis_tx(&mut recovery, &db_rw.reader, supply)
}

// Check that the genesis validators are present in the genesis blob file, once we read the db.

fn get_val_set(db_reader: &Arc<dyn DbReader>) -> anyhow::Result<Vec<AccountAddress>> {
    let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
    let root_account_state_view = db_state_view.as_account_with_state_view(&CORE_CODE_ADDRESS);

    let val_set = root_account_state_view
        .get_validator_set()
        .context("error calling get_validator_set")?
        .context("db returns None for validator set struct")?;
    Ok(val_set.payload().map(|v| *v.account_address()).collect())
}

/// get a resource type from the genesis DB
pub fn get_struct<T: MoveResource>(
    db_reader: &Arc<dyn DbReader>,
    address: Option<AccountAddress>,
) -> anyhow::Result<T> {
    let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
    let address = address.unwrap_or(CORE_CODE_ADDRESS);
    let state_view = db_state_view.as_account_with_state_view(&address);

    let resource = state_view
        .get_move_resource::<T>()
        .context("error calling get_move_resource")?
        .context("db returns None for resource struct")?;
    Ok(resource)
}

pub fn check_val_set(
    expected_vals: &[AccountAddress],
    genesis_transaction: &Transaction,
) -> Result<(), anyhow::Error> {
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(genesis_transaction)?;

    let addrs = get_val_set(&db_rw.reader)?;

    assert!(
        addrs.len() == expected_vals.len(),
        "validator set length mismatch"
    );

    for v in expected_vals {
        assert!(addrs.contains(v), "genesis does not contain validator");
    }

    Ok(())
}

pub fn check_supply(
    expected_supply: u64,
    // genesis_transaction: &Transaction,
    db_reader: &Arc<dyn DbReader>,
) -> Result<(), anyhow::Error> {
    let pb = ProgressBar::new(1000)
        .with_style(OLProgress::spinner())
        .with_message("checking coin migration");
    pb.enable_steady_tick(core::time::Duration::from_millis(500));

    let on_chain_supply = total_supply(db_reader).unwrap();

    pb.finish_and_clear();
    assert!(
        expected_supply as u128 == on_chain_supply,
        "supply mismatch, expected: {expected_supply:?} vs in genesis tx {on_chain_supply:?}"
    );
    Ok(())
}
