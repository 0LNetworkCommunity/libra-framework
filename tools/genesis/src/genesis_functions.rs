//! ol functions to run at genesis e.g. migration.

use crate::process_comm_wallet;
use anyhow::Context;
use diem_logger::prelude::*;
use diem_types::account_config::CORE_CODE_ADDRESS;
use diem_vm::move_vm_ext::SessionExt;
use diem_vm_genesis::exec_function;
use indicatif::ProgressIterator;
use libra_backwards_compatibility::legacy_recovery_v6::{AccountRole, LegacyRecoveryV6};
use libra_types::{exports::AccountAddress, ol_progress::OLProgress};

use move_core_types::value::{serialize_values, MoveValue};

/// Migrates all users' data during genesis including accounts, wallets, ancestry, and receipts.
pub fn genesis_migrate_all_users(
    session: &mut SessionExt,
    user_recovery: &mut [LegacyRecoveryV6],
) -> anyhow::Result<()> {
    user_recovery
        .iter_mut()
        .progress_with_style(OLProgress::bar())
        .for_each(|a| {
            if a.account.is_some() && a.role == AccountRole::Drop {
                warn!("Drop user, bang bang: {:?}", a.account);
                set_tombstone(session, a.account.unwrap());
                return;
            }

            if a.balance.is_none() {
                warn!("Skip migrating user, no balance: {:?}", a.account);
            }
            match genesis_migrate_one_user(session, a) {
                Ok(_) => {}
                Err(e) => {
                    // TODO: compile a list of errors.
                    if a.role != AccountRole::System {
                        error!("Error migrating user: {:?}", e);
                    }
                }
            }

            // migrating slow wallets
            if a.slow_wallet.is_some() {
                // TODO: this should not happen on a community wallet
                match genesis_migrate_slow_wallet(session, a) {
                    Ok(_) => {}
                    Err(e) => {
                        if a.role != AccountRole::System {
                            println!("Error migrating user: {:?}", e);
                        }
                    }
                }
            }

            if a.comm_wallet.is_some() {
                // migrate community wallets
                genesis_migrate_community_wallet(session, a)
                    .expect("could not migrate community wallets");
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

            if a.receipts.is_some() {
                match genesis_migrate_receipts(session, a) {
                    Ok(_) => {}
                    Err(e) => {
                        if a.role != AccountRole::System {
                            println!("Error migrating user: {:?}", e);
                        }
                    }
                }
            }

            if a.my_pledge.is_some() {
                match genesis_migrate_infra_escrow(session, a) {
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

/// Migrates data for a single user during genesis based on legacy recovery information.
pub fn genesis_migrate_one_user(
    session: &mut SessionExt,
    user_recovery: &LegacyRecoveryV6,
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

    let legacy_balance = user_recovery
        .balance
        .as_ref()
        .expect("no balance found")
        .coin;

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::vector_u8(auth_key.to_vec()),
        MoveValue::U64(legacy_balance),
    ]);

    exec_function(
        session,
        "genesis_migration",
        "migrate_legacy_user",
        vec![],
        serialized_values,
    );
    trace!("migrate_legacy_user {}", new_addr_type);
    Ok(())
}

/// Migrates slow wallet data during genesis if available.
pub fn genesis_migrate_slow_wallet(
    session: &mut SessionExt,
    user_recovery: &LegacyRecoveryV6,
    // split_factor: f64,
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

    if let Some(slow) = &user_recovery.slow_wallet {
        let serialized_values = serialize_values(&vec![
            MoveValue::Signer(CORE_CODE_ADDRESS),
            MoveValue::Signer(new_addr_type),
            MoveValue::U64(slow.unlocked),
            MoveValue::U64(slow.transferred),
        ]);

        exec_function(
            session,
            "slow_wallet",
            "fork_migrate_slow_wallet",
            vec![],
            serialized_values,
        );
    }

    Ok(())
}

/// Migrates infrastructure escrow data during genesis if available.
pub fn genesis_migrate_infra_escrow(
    session: &mut SessionExt,
    user_recovery: &LegacyRecoveryV6,
) -> anyhow::Result<()> {
    if user_recovery.account.is_none()
        || user_recovery.auth_key.is_none()
        || user_recovery.my_pledge.is_none()
    {
        anyhow::bail!("no user pledge found {:?}", user_recovery);
    }

    user_recovery
        .my_pledge
        .as_ref()
        .expect("no pledge")
        .list
        .iter()
        .for_each(|p| {
            let serialized_values = serialize_values(&vec![
                MoveValue::Signer(CORE_CODE_ADDRESS), // is sent by the 0x0 address
                MoveValue::Signer(user_recovery.account.unwrap()),
                MoveValue::U64(p.pledge),
                MoveValue::U64(p.lifetime_pledged),
                MoveValue::U64(p.lifetime_withdrawn),
            ]);

            exec_function(
                session,
                "genesis_migration",
                "fork_escrow_init",
                vec![],
                serialized_values,
            );
        });
    Ok(())
}

/// Migrates receipts data during genesis if available.
pub fn genesis_migrate_receipts(
    session: &mut SessionExt,
    user_recovery: &LegacyRecoveryV6,
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

    if let Some(receipts_vec) = user_recovery.receipts.as_ref() {
        let dest_map: Vec<AccountAddress> = receipts_vec
            .destination
            .iter()
            .map(|leg_addr| {
                AccountAddress::from_hex_literal(&format!("0x{}", leg_addr))
                    .expect("could not parse account address")
            })
            .collect();

        let cumu_map: Vec<MoveValue> = receipts_vec
            .cumulative
            .iter()
            .map(|e: &u64| MoveValue::U64(*e))
            .collect();

        let timestamp_map: Vec<MoveValue> = receipts_vec
            .last_payment_timestamp
            .iter()
            .map(|e: &u64| MoveValue::U64(*e))
            .collect();

        let payment_map: Vec<MoveValue> = receipts_vec
            .last_payment_value
            .iter()
            .map(|e: &u64| MoveValue::U64(*e))
            .collect();

        let serialized_values = serialize_values(&vec![
            MoveValue::Signer(AccountAddress::ZERO), // is sent by the 0x0 address
            MoveValue::Signer(new_addr_type),
            MoveValue::vector_address(dest_map),
            MoveValue::Vector(cumu_map),
            MoveValue::Vector(timestamp_map),
            MoveValue::Vector(payment_map),
        ]);

        exec_function(
            session,
            "receipts",
            "genesis_migrate_user",
            vec![],
            serialized_values,
        );
    }

    Ok(())
}

/// Migrates ancestry data during genesis if available.
pub fn genesis_migrate_ancestry(
    session: &mut SessionExt,
    user_recovery: &LegacyRecoveryV6,
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

            AccountAddress::from_hex_literal(&format!("0x{}", acc_str)).unwrap()
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

/// migrate the registry of Donor Voice Accounts
/// Also mark them Community Wallets if they have chosen that designation.
pub fn genesis_migrate_community_wallet(
    session: &mut SessionExt,
    user_recovery: &LegacyRecoveryV6,
) -> anyhow::Result<()> {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(user_recovery.account.unwrap()),
    ]);

    exec_function(
        session,
        "community_wallet_init",
        "migrate_community_wallet_account",
        vec![],
        serialized_values,
    );

    Ok(())
}

/// migrate the Cumulative Deposits Structs (for the Match Index weights).
pub fn genesis_migrate_cumu_deposits(
    session: &mut SessionExt,
    user_recovery: &[LegacyRecoveryV6],
) -> anyhow::Result<()> {
    let (_dr, cw) = process_comm_wallet::prepare_cw_and_receipts(user_recovery)?;

    cw.list.iter().for_each(|(addr, wallet)| {
        let serialized_values = serialize_values(&vec![
            MoveValue::Signer(CORE_CODE_ADDRESS),
            MoveValue::Signer(addr.to_owned()),
            MoveValue::U64(wallet.cumulative_value),
            MoveValue::U64(wallet.cumulative_index),
            MoveValue::vector_address(wallet.depositors.clone()),
        ]);

        exec_function(
            session,
            "cumulative_deposits",
            "genesis_migrate_cumulative_deposits",
            vec![],
            serialized_values,
        );
    });

    Ok(())
}

// before any accounts are created we need to have a FinalMint in place
// It should also happen immediately after LibraCoin gets initialized
pub fn set_final_supply(session: &mut SessionExt) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::U64(100_000_000_000u64 * 1_000_000u64), // scaled final supply
    ]);

    exec_function(
        session,
        "libra_coin",
        "genesis_set_final_supply",
        vec![],
        serialized_values,
    );
}

/// Sets the baseline reward for validators during genesis.
pub fn set_validator_baseline_reward(session: &mut SessionExt, nominal_reward: u64) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS), // must be called by 0x1
        MoveValue::U64(nominal_reward),
    ]);

    exec_function(
        session,
        "proof_of_fee",
        "genesis_migrate_reward",
        vec![],
        serialized_values,
    );
}

/// Sets a tombstone for a user account during genesis.
fn set_tombstone(session: &mut SessionExt, user: AccountAddress) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(AccountAddress::ZERO), // must be called by 0x0
        MoveValue::Address(user),
    ]);

    exec_function(
        session,
        "account",
        "set_tombstone",
        vec![],
        serialized_values,
    );
}
