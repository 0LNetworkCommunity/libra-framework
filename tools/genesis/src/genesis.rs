//! create a genesis from a LegacyRecovery struct

use crate::vm::zapatos_mainnet_genesis;
use anyhow::Error;
use ol_types::legacy_recovery::LegacyRecovery;
use zapatos_framework::ReleaseBundle;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use zapatos_types::{transaction::{Transaction, WriteSetPayload}, chain_id::ChainId};
use zapatos_vm_genesis::Validator;
/// Make a recovery genesis blob
pub fn make_recovery_genesis_from_vec_legacy_recovery(
    recovery: &[LegacyRecovery],
    genesis_vals: &[Validator],
    // genesis_blob_path: &PathBuf,
    framework_release: &ReleaseBundle,
    chain_id: ChainId,
    // append_user_accounts: bool,
) -> Result<Transaction, Error> {
    dbg!(&"make_recovery_genesis_from_vec_legacy_recovery");
    // // get consensus accounts
    // let all_validator_configs = legacy_recovery::recover_validator_configs(recovery)?;

    // // check the validators that are joining genesis actually have legacy data
    // let count = all_validator_configs.vals
    // .iter()
    // .filter(
    //   |v| {
    //     dbg!(&v.val_account);
    //     // let string_addr = v.val_account.to_string();
    //     // let addr = AccountAddress::from_hex_literal(&string_addr).unwrap();
    //     // genesis_vals.contains(&addr)
    //     true
    //   }
    // )
    // .count();

    // if count == 0 {
    //   anyhow::bail!("no val configs found for genesis set");
    // }

    let recovery_changeset = zapatos_mainnet_genesis(genesis_vals, Some(recovery), framework_release, chain_id)?;

    // For a real upgrade or fork, we want to include all user accounts.
    // this is the default.
    // Otherwise, we might need to just collect the validator accounts
    // for debugging or other test purposes.
    // let expected_len_all_users = recovery.len() as u64;

    let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset));

    // save genesis
    // save_genesis(&gen_tx, genesis_blob_path)?;
    // todo!()
    Ok(gen_tx)
}

/// save the genesis blob
pub fn save_genesis(gen_tx: &Transaction, output_path: &PathBuf) -> Result<(), Error> {
    // let file_path = output_path.join("genesis").with_extension("blob");
    let mut file = File::create(output_path)?;
    let bytes = bcs::to_bytes(&gen_tx)?;
    file.write_all(&bytes)?;
    Ok(())
}
