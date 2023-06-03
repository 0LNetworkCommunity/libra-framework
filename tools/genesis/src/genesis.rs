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
    recovery: Option<&[LegacyRecovery]>,
    genesis_vals: &[Validator],
    // genesis_blob_path: &PathBuf,
    framework_release: &ReleaseBundle,
    chain_id: ChainId,
    // append_user_accounts: bool,
) -> Result<Transaction, Error> {
    dbg!(&"make_recovery_genesis_from_vec_legacy_recovery");

    // Note: For `recovery` on a real upgrade or fork, we want to include all user accounts. If a None is passed, then we'll just run the default genesis
    // which only uses the validator accounts.
    let recovery_changeset = zapatos_mainnet_genesis(genesis_vals, recovery, framework_release, chain_id)?;



    let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(recovery_changeset));

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
