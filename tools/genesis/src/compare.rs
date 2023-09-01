//! functions for comparing LegacyRecovery data to a genesis blob
//!
//! every day is like sunday
//! -- morrissey via github copilot
use crate::genesis_reader;
use crate::genesis_reader::total_supply;
use crate::supply::Supply;

use anyhow;
// use libra_types::move_resource::coin_info::GasCoinInfoResource;
use libra_types::exports::AccountAddress;
use libra_types::legacy_types::legacy_address::LegacyAddress;
use libra_types::legacy_types::legacy_recovery::{read_from_recovery_file, LegacyRecovery};
use libra_types::move_resource::gas_coin::{GasCoinStoreResource, SlowWalletBalance};
use libra_types::ol_progress::OLProgress;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use zapatos_storage_interface::state_view::LatestDbStateCheckpointView;
use zapatos_types::transaction::Transaction;

use indicatif::{ProgressBar, ProgressIterator};
use std::path::PathBuf;
use std::sync::Arc;
use zapatos_state_view::account_with_state_view::AsAccountWithStateView;
use zapatos_storage_interface::DbReader;
use zapatos_types::account_view::AccountView;

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
    genesis_transaction: &Transaction,
    supply: &Supply,
) -> Result<Vec<CompareError>, anyhow::Error> {
    // start an empty btree map
    let mut err_list: Vec<CompareError> = vec![];

    let pb = ProgressBar::new(1000)
        .with_style(OLProgress::spinner())
        .with_message("check genesis bootstraps db");
    pb.enable_steady_tick(core::time::Duration::from_millis(500));
    // iterate over the recovery file and compare balances
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(genesis_transaction)?;
    pb.finish_and_clear();

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

            let db_state_view = db_rw.reader.latest_state_checkpoint_view().unwrap();
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

                let expected_balance = supply.split_factor * balance_legacy.coin as f64;

                if on_chain_balance.coin() != expected_balance as u64 {
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
    compare_recovery_vec_to_genesis_tx(&recovery, &gen_tx, supply)
}

// Check that the genesis validators are present in the genesis blob file, once we read the db.

fn get_val_set(db_reader: &Arc<dyn DbReader>) -> anyhow::Result<Vec<AccountAddress>> {
    let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
    let root_account_state_view = db_state_view.as_account_with_state_view(&CORE_CODE_ADDRESS);

    let val_set = root_account_state_view
        .get_validator_set()
        .unwrap()
        .unwrap();
    Ok(val_set.payload().map(|v| *v.account_address()).collect())
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
    genesis_transaction: &Transaction,
) -> Result<(), anyhow::Error> {
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(genesis_transaction)?;

    let on_chain_supply = total_supply(&db_rw.reader).unwrap();

    assert!(
        expected_supply as u128 == on_chain_supply,
        "supply mismatch, expected: {expected_supply:?} vs in genesis tx {on_chain_supply:?}"
    );

    Ok(())
}
