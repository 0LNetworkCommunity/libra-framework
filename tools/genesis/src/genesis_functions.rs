//! ol functions to run at genesis e.g. migration.
use crate::hack_cli_progress::OLProgress;
use crate::supply::get_supply_struct;
use anyhow::Context;
use indicatif::ProgressIterator;
use libra_types::exports::AccountAddress;
use libra_types::legacy_types::{
    legacy_address::LegacyAddress,
    legacy_recovery::{AccountRole, LegacyRecovery},
};
use move_core_types::{
    resolver::MoveResolver,
    value::{serialize_values, MoveValue},
};
use zapatos_types::account_config::CORE_CODE_ADDRESS;
use zapatos_vm::move_vm_ext::SessionExt;
use zapatos_vm_genesis::exec_function;

pub fn genesis_migrate_all_users(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &[LegacyRecovery],
) -> anyhow::Result<()> {
    // donor directed addresses that should be liquided before upgrade
    // the sample would be distorted by including these
    let ignore_for_dd_count = vec![
        // FTW
        "3A6C51A0B786D644590E8A21591FA8E2"
            .parse::<LegacyAddress>()
            .unwrap(),
        // tip jar
        "2B0E8325DEA5BE93D856CFDE2D0CBA12"
            .parse::<LegacyAddress>()
            .unwrap(),
    ];

    let supply = get_supply_struct(user_recovery, ignore_for_dd_count)?;
    let split_factor = 10_000_000_000.0 / supply.total;

    let target_future_uses = supply.total * 0.70;
    let remaining_to_fund = target_future_uses - supply.donor_directed;
    let escrow_pct: f64 = remaining_to_fund / supply.slow_validator_locked;

    user_recovery
        .iter()
        .progress_with_style(OLProgress::bar())
        .for_each(|a| {
            match genesis_migrate_one_user(session, &a, split_factor, escrow_pct) {
                Ok(_) => {}
                Err(e) => {
                    // TODO: compile a list of errors.
                    println!("Error migrating user: {:?}", e);
                }
            }
        });
    Ok(())
}

pub fn genesis_migrate_one_user(
    //////// 0L ////////
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
    split_factor: f64,
    escrow_pct: f64,
) -> anyhow::Result<()> {

    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.balance.is_none()
    {
        anyhow::bail!("no user account found");
    }

    // convert between different types from ol_types in diem, to current
    let acc_str = user_recovery
        .account
        .context("could not parse account")?
        .to_string();
    let new_addr_type = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))?;

    // NOTE: Authkeys have the same format as in pre V7
    let auth_key = user_recovery.auth_key.context("no auth key found")?;

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::vector_u8(auth_key.to_vec()),
        MoveValue::U64(
            user_recovery
                .balance
                .as_ref()
                .expect("no balance found")
                .coin,
        ),
        MoveValue::Bool(user_recovery.role == AccountRole::Validator),
        MoveValue::U64((split_factor * 1_000_000.0) as u64),
        MoveValue::U64((escrow_pct * 1_000_000.0) as u64),
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
