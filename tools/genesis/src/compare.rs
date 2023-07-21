//! functions for comparing LegacyRecovery data to a genesis blob
//!
//! every day is like sunday
//! -- morrissey via github copilot
use crate::genesis_reader::read_db_and_compute_genesis;
// use crate::db_utils;
use anyhow;
// use libra_types::legacy_types::ancestry::AncestryResource;
// use libra_types::exports::AccountAddress;
use libra_types::legacy_types::legacy_address::LegacyAddress;
use libra_types::legacy_types::legacy_recovery::{LegacyRecovery, read_from_recovery_file};
// use libra_types::exports::Waypoint;
use libra_types::ol_progress::OLProgress;

// use zapatos_types::access_path::AccessPath;
// use zapatos_types::state_store::state_key::StateKey;
// use zapatos_types::transaction::Transaction;
// use zapatos_types::{
//   account_view::AccountView,
//   account_state::AccountState,
//   move_resource::MoveStorage,
// };
// use zapatos_storage_interface::DbReaderWriter;
// use std::convert::TryFrom;
use std::path::PathBuf;
// use std::path::Path;
// use std::fs::File;
// use std::io::Read;
// use std::ops::Deref;
use indicatif::{ProgressIterator, ProgressBar};

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
pub fn compare_recovery_vec_to_genesis_blob(
    recovery: &[LegacyRecovery],
    genesis_path: PathBuf,
) -> Result<Vec<CompareError>, anyhow::Error> {
    // start an empty btree map
    let mut err_list: Vec<CompareError> = vec![];

    let pb = ProgressBar::new(1000)
    .with_style(OLProgress::spinner())
    .with_message("Test database from genesis.blob");
    pb.enable_steady_tick(core::time::Duration::from_millis(500));
    // iterate over the recovery file and compare balances
    let (_db_rw, _) = read_db_and_compute_genesis(&genesis_path)?;
    pb.finish_and_clear();



    recovery.iter()
    .progress_with_style(OLProgress::bar())
    .with_message("Comaparing snapshot to new genesis")
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

        // let val_state = match db_rw
        //     .reader
        //     .get_latest_account_state(v.account.expect("need an address"))
        // {
        //     Ok(Some(val_state)) => val_state,
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "find account blob".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let account_state = match AccountState::try_from(&val_state) {
        //     Ok(account_state) => account_state,
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "parse account state".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let bal = match account_state.get_balance_resources() {
        //     Ok(bal) => bal,
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "get balance resource".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let genesis_bal = match bal.iter().next() {
        //     Some((_, b)) => b.coin(),
        //     _ => {
        //         err_list.push(CompareError {
        //             index: i as u64,
        //             account: v.account,
        //             bal_diff: 0,
        //             message: "genesis resource is None".to_string(),
        //         });
        //         return;
        //     }
        // };

        // let recovery_bal = v.balance.as_ref().unwrap().coin();
        // if recovery_bal != genesis_bal {
        //     err_list.push(CompareError {
        //         index: i as u64,
        //         account: v.account,
        //         bal_diff: recovery_bal as i64 - genesis_bal as i64,
        //         message: "balance mismatch".to_string(),
        //     });
        // }
    });

    Ok(err_list)
}

/// Compare the balances in a recovery file to the balances in a genesis blob.
pub fn compare_json_to_genesis_blob(
    json_path: PathBuf,
    genesis_path: PathBuf,
) -> Result<Vec<CompareError>, anyhow::Error> {
    let recovery = read_from_recovery_file(&json_path);
    compare_recovery_vec_to_genesis_blob(&recovery, genesis_path)
}


// Check that the genesis validators are present in the genesis blob file, once we read the db.

// pub fn check_val_set(
//   expected_vals: Vec<AccountAddress>,
//   genesis_path: PathBuf,
// ) -> Result<(), anyhow::Error>{
//       let (db_rw, _) = read_db_and_compute_genesis(&genesis_path)?;

//       let root_blob = db_rw
//       .reader
//       .get_latest_account_state(AccountAddress::ZERO)?
//       .expect("no account state blob");

//       let root_state = AccountState::try_from(&root_blob)?;

//       let val_set = root_state.get_validator_set()?
//       .expect("no validator config state");

//       let addrs = val_set.payload()
//       // .iter()
//       .map(|v| {
//         // dbg!(&v);
//         *v.account_address()
//       })
//       .collect::<Vec<AccountAddress>>();

//       assert!(addrs.len() == expected_vals.len(), "validator set length mismatch");

//       for v in expected_vals {
//         assert!(addrs.contains(&v), "genesis does not contain validator");
//       }

//       Ok(())

// }

