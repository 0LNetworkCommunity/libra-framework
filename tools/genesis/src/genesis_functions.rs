//! ol functions to run at genesis e.g. migration.
use crate::supply::{populate_supply_stats_from_legacy, SupplySettings};
use anyhow::Context;
use indicatif::ProgressIterator;
use libra_types::{
    exports::AccountAddress,
    legacy_types::legacy_recovery::{AccountRole, LegacyRecovery},
    ol_progress::OLProgress,
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
    supply_settings: &SupplySettings,
) -> anyhow::Result<()> {
    let mut supply = populate_supply_stats_from_legacy(user_recovery, &supply_settings.map_dd_to_slow)?;

    supply.set_ratios_from_settings(supply_settings)?;

    user_recovery
        .iter()
        .progress_with_style(OLProgress::bar())
        .for_each(|a| {
            // do basic account creation and coin scaling
            match genesis_migrate_one_user(session, &a, supply.split_factor, supply.escrow_pct) {
                Ok(_) => {}
                Err(e) => {
                    // TODO: compile a list of errors.
                    if a.role != AccountRole::System {
                        println!("Error migrating user: {:?}", e);
                    }
                }
            }

            // migrating slow wallets
            if a.slow_wallet.is_some() {
                match genesis_migrate_slow_wallet(session, a, supply.split_factor) {
                    Ok(_) => {}
                    Err(e) => {
                        if a.role != AccountRole::System {
                            println!("Error migrating user: {:?}", e);
                        }
                    }
                }
            }

            // migrating infra escrow, check if this has historically been a validator and has a slow wallet
            if a.val_cfg.is_some() && a.slow_wallet.is_some() {
                match genesis_migrate_infra_escrow(session, a, supply.escrow_pct) {
                    Ok(_) => {}
                    Err(e) => {
                        if a.role != AccountRole::System {
                            println!("Error migrating user: {:?}", e);
                        }
                    }
                }
            }

            // migrating ancestry
            if a.ancestry.is_some() {
                match genesis_migrate_ancestry(session, a) {
                    Ok(_) => {}
                    Err(e) => {
                        if a.role != AccountRole::System {
                            println!("Error migrating user: {:?}", e);
                        }
                    }
                }
            }

            // migrating tower
            if a.miner_state.is_some() {
                match genesis_migrate_tower_state(session, a) {
                    Ok(_) => {}
                    Err(e) => {
                        if a.role != AccountRole::System {
                            println!("Error migrating user: {:?}", e);
                        }
                    }
                }
            }
        });
    Ok(())
}

pub fn genesis_migrate_one_user(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
    split_factor: f64,
    escrow_pct: f64,
) -> anyhow::Result<()> {
    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.balance.is_none()
    {
        anyhow::bail!("no user account found {:?}", user_recovery);
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

pub fn genesis_migrate_slow_wallet(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
    split_factor: f64,
) -> anyhow::Result<()> {
    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.balance.is_none()
        || user_recovery.slow_wallet.is_none()
    {
        anyhow::bail!("no user account found {:?}", user_recovery);
    }

    // convert between different types from ol_types in diem, to current
    let acc_str = user_recovery
        .account
        .context("could not parse account")?
        .to_string();
    let new_addr_type = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))?;

    let slow = user_recovery.slow_wallet.as_ref().unwrap();
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::U64(slow.unlocked),
        MoveValue::U64(slow.transferred),
        MoveValue::U64((split_factor * 1_000_000.0) as u64),
    ]);

    exec_function(
        session,
        "slow_wallet",
        "fork_migrate_slow_wallet",
        vec![],
        serialized_values,
    );
    Ok(())
}

pub fn genesis_migrate_infra_escrow(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
    escrow_pct: f64,
) -> anyhow::Result<()> {
    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.balance.is_none()
        || user_recovery.slow_wallet.is_none()
    {
        anyhow::bail!("no user account found {:?}", user_recovery);
    }

    // convert between different types from ol_types in diem, to current
    let acc_str = user_recovery
        .account
        .context("could not parse account")?
        .to_string();
    let new_addr_type = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))?;

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::U64((escrow_pct * 1_000_000.0) as u64),
    ]);

    exec_function(
        session,
        "infra_escrow",
        "fork_escrow_init",
        vec![],
        serialized_values,
    );
    Ok(())
}

pub fn genesis_migrate_tower_state(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
) -> anyhow::Result<()> {
    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.balance.is_none()
        || user_recovery.miner_state.is_none()
    {
        anyhow::bail!("no user account found {:?}", user_recovery);
    }

    // convert between different types from ol_types in diem, to current
    let acc_str = user_recovery
        .account
        .context("could not parse account")?
        .to_string();
    let new_addr_type = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))?;

    let miner = user_recovery.miner_state.as_ref().unwrap();

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::vector_u8(miner.previous_proof_hash.to_vec()),
        MoveValue::U64(miner.verified_tower_height),
        MoveValue::U64(miner.latest_epoch_mining),
        MoveValue::U64(miner.count_proofs_in_epoch),
        MoveValue::U64(miner.epochs_validating_and_mining),
        MoveValue::U64(miner.contiguous_epochs_validating_and_mining),
    ]);

    exec_function(
        session,
        "tower_state",
        "fork_migrate_user_tower_history",
        vec![],
        serialized_values,
    );
    Ok(())
}

pub fn genesis_migrate_ancestry(
    session: &mut SessionExt<impl MoveResolver>,
    user_recovery: &LegacyRecovery,
) -> anyhow::Result<()> {
    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.balance.is_none()
        || user_recovery.ancestry.is_none()
    {
        anyhow::bail!("no user account found {:?}", user_recovery);
    }

    // convert between different types from ol_types in diem, to current
    let acc_str = user_recovery
        .account
        .context("could not parse account")?
        .to_string();
    let new_addr_type = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))?;

    let ancestry = user_recovery.ancestry.as_ref().unwrap();

    let mapped_addr: Vec<AccountAddress> = ancestry
        .tree
        .iter()
        .map(|el| {
            let acc_str = el.to_string();
            let new_addr_type =
                AccountAddress::from_hex_literal(&format!("0x{}", acc_str)).unwrap();
            new_addr_type
        })
        .collect();

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::vector_address(mapped_addr),
    ]);

    exec_function(
        session,
        "ancestry",
        "fork_migrate",
        vec![],
        serialized_values,
    );
    Ok(())
}
