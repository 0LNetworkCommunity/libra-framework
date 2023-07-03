//! ol functions to run at genesis e.g. migration.
use crate::hack_cli_progress::OLProgress;
// use libra_wallet::legacy::convert_legacy::convert_account;
use anyhow::Context;
use zapatos_types::account_config::CORE_CODE_ADDRESS;
use zapatos_vm::{
    move_vm_ext::SessionExt,
};
use move_core_types::{
    resolver::MoveResolver,
        value::{serialize_values, MoveValue},

};
use ol_types::legacy_recovery::{LegacyRecovery, AccountRole};
use libra_types::exports::AccountAddress;
use zapatos_vm_genesis::exec_function;
use indicatif::ProgressIterator;


pub fn genesis_migrate_all_users(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &[LegacyRecovery],
) -> anyhow::Result<()>{


  user_recovery.iter().progress_with_style(OLProgress::bar()).for_each(|a| {
    match genesis_migrate_one_user(session, &a) {
        Ok(_) => {},
        Err(e) => {
            // TODO: compile a list of errors.
            println!("Error migrating user: {:?}", e);
      }
    }
  });
  Ok(())
}

pub fn genesis_migrate_one_user( //////// 0L ////////
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
) -> anyhow::Result<()> {
    // let validators_bytes = bcs::to_bytes(validators).expect("Validators can be serialized");
    // let mut serialized_values = serialize_values(&vec![MoveValue::Signer(CORE_CODE_ADDRESS)]);
      // vm: &signer,
      // user_sig: &signer,
      // auth_key: vector<u8>,
      // legacy_balance: u64,
      // is_validator: bool,

    if user_recovery.account.is_none() ||
        user_recovery.auth_key.is_none() ||
        user_recovery.balance.is_none() {
        anyhow::bail!("no user account found");
    }

    // let new_addr_type = convert_account(user_recovery.account.expect("no account found"))?;
    let acc_str = user_recovery.account.context("could not parse account")?.to_string();
    let new_addr_type = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))?;
    // let new_auth_type = convert_auth_key(user_recovery.auth_key.expect("no auth key found"))?;
    // NOTE: Authkeys have the same format as in pre V7
    let auth_key = user_recovery.auth_key.context("no auth key found")?;

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::vector_u8(auth_key.to_vec()),
        MoveValue::U64(user_recovery.balance.as_ref().expect("no balance found").coin()),
        MoveValue::Bool(user_recovery.role == AccountRole::Validator),
    ]);

    exec_function(
        session,
        "genesis_migration",
        "migrate_legacy_user",
        vec![],
        serialized_values,
    );
    Ok(())
}