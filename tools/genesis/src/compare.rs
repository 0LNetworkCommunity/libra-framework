//! functions for comparing LegacyRecovery data to a genesis blob
//!
//! every day is like sunday
//! -- morrissey via github copilot
use crate::genesis_reader;
use crate::genesis_reader::total_supply;
use crate::supply::Supply;

use anyhow::{self, Context};
// use libra_types::move_resource::coin_info::GasCoinInfoResource;
use diem_storage_interface::state_view::LatestDbStateCheckpointView;
use diem_types::transaction::Transaction;
use libra_types::exports::AccountAddress;
use libra_types::legacy_types::legacy_address::LegacyAddress;
use libra_types::legacy_types::legacy_recovery::{read_from_recovery_file, LegacyRecovery};
use libra_types::move_resource::gas_coin::{GasCoinStoreResource, SlowWalletBalance};
use libra_types::ol_progress::OLProgress;
use move_core_types::language_storage::CORE_CODE_ADDRESS;

use diem_state_view::account_with_state_view::AsAccountWithStateView;
use diem_storage_interface::DbReader;
use diem_types::account_view::AccountView;
use indicatif::{ProgressBar, ProgressIterator};
use move_core_types::move_resource::MoveResource;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
/// struct for holding the results of a comparison
pub struct CompareError {
    /// index of LegacyRecover
    pub index: u64,
    /// user account
    pub account: Option<LegacyAddress>,
    /// balance difference [LegacyRecover]- [genesis blob]
    pub bal_diff: i64,
    /// error message
    pub message: String,
}
/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_recovery_vec_to_genesis_tx(
    recovery: &[LegacyRecovery],
    db_reader: &Arc<dyn DbReader>,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    // start an empty btree map
    let mut err_list: Vec<CompareError> = vec![];

    recovery
        .iter()
        .progress_with_style(OLProgress::bar())
        .with_message("auditing migration")
        .enumerate()
        .for_each(|(i, v)| {
            if v.account.is_none() {
                err_list.push(CompareError {
                    index: i as u64,
                    account: None,
                    bal_diff: 0,
                    message: "account is None".to_string(),
                }); // instead of balance, if there is an account that is None, we insert the index of the recovery file
                return;
            };

            if v.account.unwrap() == LegacyAddress::ZERO {
                return;
            };
            if v.balance.is_none() {
                err_list.push(CompareError {
                    index: i as u64,
                    account: v.account,
                    bal_diff: 0,
                    message: "recovery file balance is None".to_string(),
                });
                return;
            }

            let convert_address =
                AccountAddress::from_hex_literal(&v.account.unwrap().to_hex_literal())
                    .expect("could not convert address types");

            let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
            let account_state_view = db_state_view.as_account_with_state_view(&convert_address);

            if let Some(slow_legacy) = &v.slow_wallet {
                let on_chain_slow_wallet = account_state_view
                    .get_move_resource::<SlowWalletBalance>()
                    .expect("should have a slow wallet struct")
                    .unwrap();

                let expected_unlocked = supply.split_factor * slow_legacy.unlocked as f64;

                if on_chain_slow_wallet.unlocked != expected_unlocked as u64 {
                    err_list.push(CompareError {
                        index: i as u64,
                        account: v.account,
                        bal_diff: on_chain_slow_wallet.unlocked as i64 - expected_unlocked as i64,
                        message: "unexpected slow wallet unlocked".to_string(),
                    });
                }
            }

            if let Some(balance_legacy) = &v.balance {
                let on_chain_balance = account_state_view
                    .get_move_resource::<GasCoinStoreResource>()
                    .expect("should have move resource")
                    .expect("should have a GasCoinStoreResource for balance");

                let expected_balance = if v.val_cfg.is_some() && v.slow_wallet.is_some() {
                  let unlocked = v.slow_wallet.as_ref().unwrap().unlocked;
                  let val_locked = balance_legacy.coin - unlocked;

                  let val_pledge = (val_locked as f64) * supply.escrow_pct;

                  let total_balance = (val_locked as f64 - val_pledge) + unlocked as f64;

                  // scale it
                  total_balance * supply.split_factor

                } else {
                  supply.split_factor * balance_legacy.coin as f64
                };

                if on_chain_balance.coin() != expected_balance as u64 {

                    dbg!(&on_chain_balance.coin(), &expected_balance);
                    err_list.push(CompareError {
                        index: i as u64,
                        account: v.account,
                        bal_diff: on_chain_balance.coin() as i64 - expected_balance as i64,
                        message: "unexpected balance".to_string(),
                    });
                }
            }
        });

    Ok(err_list)
}

/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_json_to_genesis_blob(
    json_path: PathBuf,
    genesis_path: PathBuf,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    let recovery = read_from_recovery_file(&json_path);

    let gen_tx = genesis_reader::read_blob_to_tx(genesis_path)?;
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx)?;
    compare_recovery_vec_to_genesis_tx(&recovery, &db_rw.reader, supply)
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
