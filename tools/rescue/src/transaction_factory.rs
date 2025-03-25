use crate::session_tools;
use anyhow::Result;
use diem_types::{
    account_address::AccountAddress,
    transaction::{Script, Transaction, WriteSetPayload},
};
use libra_config::validator_registration::{registration_from_operator_yaml, ValCredentials};
use libra_framework::builder::framework_generate_upgrade_proposal::libra_compile_script;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use std::path::{Path, PathBuf};

pub fn save_rescue_blob(tx: Transaction, out_file: &Path) -> Result<PathBuf> {
    let bytes = bcs::to_bytes(&tx)?;
    std::fs::write(out_file, bytes.as_slice())?;
    Ok(out_file.to_path_buf())
}

pub fn run_script_tx(script_path: &Path) -> Result<Transaction> {
    println!("Running script from {:?}", script_path);
    println!("The script must use only functions available installed in the reference db's framework (not what is in the repo source).");
    let (code, _hash) = libra_compile_script(script_path, false)?;

    let wp = WriteSetPayload::Script {
        execute_as: CORE_CODE_ADDRESS,
        script: Script::new(code, vec![], vec![]),
    };

    Ok(Transaction::GenesisTransaction(wp))
}

pub fn upgrade_tx(
    db_path: &Path,
    upgrade_mrb: &Path,
    validator_set: Option<Vec<AccountAddress>>,
) -> Result<Transaction> {
    let cs = session_tools::upgrade_framework_changeset(db_path, validator_set, upgrade_mrb)?;
    Ok(Transaction::GenesisTransaction(WriteSetPayload::Direct(cs)))
}

pub fn register_vals(
    db_path: &Path,
    reg_files: &[PathBuf],
    upgrade_mrb: &Option<PathBuf>,
) -> Result<Transaction> {
    // TODO: replace ValCredentials with OperatorConfiguration
    let registrations: Vec<ValCredentials> = reg_files
        .iter()
        .map(|el| {
            registration_from_operator_yaml(Some(el.to_path_buf()))
                .expect("could parse operator.yaml")
        })
        .collect();

    let cs = session_tools::register_and_replace_validators_changeset(
        db_path,
        registrations,
        upgrade_mrb,
    )?;
    Ok(Transaction::GenesisTransaction(WriteSetPayload::Direct(cs)))
}
