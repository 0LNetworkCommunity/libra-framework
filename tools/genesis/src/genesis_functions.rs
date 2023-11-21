//! ol functions to run at genesis e.g. migration.
use crate::{
    process_comm_wallet,
    supply::{Supply, SupplySettings},
};

use anyhow::Context;
use diem_logger::prelude::*;
use diem_types::account_config::CORE_CODE_ADDRESS;
use diem_vm::move_vm_ext::SessionExt;
use diem_vm_genesis::exec_function;
use indicatif::ProgressIterator;
use libra_types::{
    exports::AccountAddress,
    legacy_types::legacy_recovery::{AccountRole, LegacyRecovery},
    ol_progress::OLProgress,
};
use move_core_types::value::{serialize_values, MoveValue};

pub fn genesis_migrate_all_users(
    session: &mut SessionExt,
    user_recovery: &[LegacyRecovery],
    supply: &Supply,
) -> anyhow::Result<()> {
    user_recovery
        .iter()
        .progress_with_style(OLProgress::bar())
        .for_each(|a| {
            // do basic account creation and coin scaling
            match genesis_migrate_one_user(session, a, supply.split_factor) {
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
                // TODO: this should not happen on a community wallet
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

            // migrating tower
            if a.receipts.is_some() {
                match genesis_migrate_receipts(session, a, supply) {
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
    session: &mut SessionExt,
    user_recovery: &LegacyRecovery,
    split_factor: f64,
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

    let rescaled_balance = (split_factor * legacy_balance as f64) as u64;

    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::Signer(new_addr_type),
        MoveValue::vector_u8(auth_key.to_vec()),
        MoveValue::U64(rescaled_balance),
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

pub fn genesis_migrate_slow_wallet(
    session: &mut SessionExt,
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
        MoveValue::U64((slow.unlocked as f64 * split_factor) as u64),
        MoveValue::U64((slow.transferred as f64 * split_factor) as u64),
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
    session: &mut SessionExt,
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
        MoveValue::Signer(AccountAddress::ZERO), // is sent by the 0x0 address
        MoveValue::Signer(new_addr_type),
        MoveValue::U64((escrow_pct * 1_000_000.0) as u64),
    ]);

    exec_function(
        session,
        "genesis_migration",
        "fork_escrow_init",
        vec![],
        serialized_values,
    );
    Ok(())
}

pub fn genesis_migrate_receipts(
    session: &mut SessionExt,
    user_recovery: &LegacyRecovery,
    supply: &Supply,
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
            .map(|e| {
                let scaled = (*e as f64) * supply.split_factor;
                MoveValue::U64(scaled as u64)
            })
            .collect();

        let timestamp_map: Vec<MoveValue> = receipts_vec
            .last_payment_timestamp
            .iter()
            .map(|e| MoveValue::U64(*e))
            .collect();

        let payment_map: Vec<MoveValue> = receipts_vec
            .last_payment_value
            .iter()
            .map(|e| {
                let scaled = (*e as f64) * supply.split_factor;
                MoveValue::U64(scaled as u64)
            })
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

pub fn genesis_migrate_tower_state(
    session: &mut SessionExt,
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
    session: &mut SessionExt,
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
    user_recovery: &[LegacyRecovery],
) -> anyhow::Result<()> {
    if let Some(root) = user_recovery.iter().find(|e| e.role == AccountRole::System) {
        let cw_list = &root
            .comm_wallet
            .as_ref()
            .context("no comm_wallet struct")?
            .list;

        cw_list.iter().for_each(|el| {
            let acc_str = el.to_string();

            let new_address = AccountAddress::from_hex_literal(&format!("0x{}", acc_str))
                .expect("could not parse address");

            let serialized_values = serialize_values(&vec![
                MoveValue::Signer(CORE_CODE_ADDRESS),
                MoveValue::Signer(new_address),
            ]);

            exec_function(
                session,
                "community_wallet",
                "migrate_community_wallet_account",
                vec![],
                serialized_values,
            );
        });
    }

    Ok(())
}

/// migrate the Cumulative Deposits Structs (for the Match Index weights).
pub fn genesis_migrate_cumu_deposits(
    session: &mut SessionExt,
    user_recovery: &[LegacyRecovery],
    split_factor: f64,
) -> anyhow::Result<()> {
    let (_dr, cw) = process_comm_wallet::prepare_cw_and_receipts(user_recovery, split_factor)?;

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
/// Since we are minting for each account to convert account balances there may be a rounding difference from target. Add those micro cents into the transaction fee account.
/// Note: we could have minted one giant coin and then split it, however there's no place to store in on chain without repurposing accounts (ie. system accounts by design do no hold any funds, only the transaction_fee contract can temporarily hold an aggregatable coin which by design can only be fully withdrawn (not split)). So the rounding mint is less elegant, but practical.
pub fn rounding_mint(session: &mut SessionExt, supply_settings: &SupplySettings) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::U64(supply_settings.scale_supply() as u64),
    ]);

    exec_function(
        session,
        "genesis_migration",
        "rounding_mint",
        vec![],
        serialized_values,
    );
}

// before any accounts are created we need to have a FinalMint in place
// It should also happen immediately after LibraCoin gets initialized
pub fn set_final_supply(session: &mut SessionExt, supply_settings: &SupplySettings) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(CORE_CODE_ADDRESS),
        MoveValue::U64(supply_settings.scale_supply() as u64),
    ]);

    exec_function(
        session,
        "libra_coin",
        "genesis_set_final_supply",
        vec![],
        serialized_values,
    );
}

pub fn set_validator_baseline_reward(session: &mut SessionExt, nominal_reward: u64) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(AccountAddress::ZERO), // must be called by 0x0
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

pub fn create_make_whole_incident(
    session: &mut SessionExt,
    user_recovery: &[LegacyRecovery],
    make_whole_budget: f64,
    split_factor: f64,
) -> anyhow::Result<()> {
    let scaled_budget = (make_whole_budget * split_factor) as u64;
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(AccountAddress::ZERO), // must be called by 0x0
        MoveValue::U64(scaled_budget),
    ]);

    exec_function(
        session,
        "genesis_migration",
        "init_make_whole",
        vec![],
        serialized_values,
    );

    user_recovery
        .iter()
        .progress_with_style(OLProgress::bar())
        .for_each(|a| {
            if let Some(mk) = &a.make_whole {
                let user_coins = mk.credits.iter().fold(0, |sum, i| i.coins.value + sum);
                create_make_whole_each_user_credit(
                    session,
                    a.account
                        .expect("could not find accout")
                        .try_into()
                        .unwrap(),
                    (user_coins as f64 * split_factor) as u64,
                )
            }
        });
    Ok(())
}

fn create_make_whole_each_user_credit(session: &mut SessionExt, user: AccountAddress, value: u64) {
    let serialized_values = serialize_values(&vec![
        MoveValue::Signer(AccountAddress::ZERO), // must be called by 0x0
        MoveValue::Address(user),
        MoveValue::U64(value),
    ]);

    exec_function(
        session,
        "genesis_migration",
        "vm_create_credit_user",
        vec![],
        serialized_values,
    );
}
