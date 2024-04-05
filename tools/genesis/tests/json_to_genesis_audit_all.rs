//! Tests for the `make_genesis` binary.
mod support;
use diem_types::chain_id::NamedChain;
use libra_framework::head_release_bundle;
use libra_genesis_tools::parse_json::recovery_file_parse;
use libra_genesis_tools::supply::{self};
use libra_genesis_tools::vm::libra_genesis_default;
use libra_genesis_tools::{compare, genesis::make_recovery_genesis_from_vec_legacy_recovery};
use libra_genesis_tools::{genesis_reader, parse_json};
use libra_types::exports::ChainId;
use support::{path_utils::json_path, test_vals};

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_correct_supply_arithmetic_all() {
    let genesis_vals = test_vals::get_test_valset(4);

    let path = json_path()
        .parent()
        .unwrap()
        .join("sample_export_recovery.json");

    let mut user_accounts = recovery_file_parse(path).unwrap();

    // get the supply arithmetic so that we can compare outputs
    let supply_stats = supply::populate_supply_stats_from_legacy(&user_accounts).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::mainnet(),
        &libra_genesis_default(NamedChain::MAINNET),
    )
    .unwrap();

    // NOTE: in the case of a single account being migrated, that account balance will equal the total supply as set in: SupplySettings. i.e. 10B
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx).unwrap();

    // LEAVE THIS CODE in case we need to dump this artifact.
    // test dump balances
    // compare::export_account_balances(&user_accounts, &db_rw.reader, json_path().parent().unwrap())
    //     .unwrap();

    // audit
    match compare::compare_recovery_vec_to_genesis_tx(
        &mut user_accounts,
        &db_rw.reader,
        &supply_stats,
    ) {
        Ok(list) => {
            if !list.is_empty() {
                let len = list.len();
                let out = json_path().parent().unwrap().join("audit.json");
                std::fs::write(out, serde_json::to_string_pretty(&list).unwrap()).unwrap();
                panic!("audit list is not empty, errs: {len}");
            }
        }
        Err(_e) => panic!("error creating comparison"),
    }
}

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_drop_all() {
    let genesis_vals = test_vals::get_test_valset(4);

    let path = json_path()
        .parent()
        .unwrap()
        .join("sample_export_recovery.json");

    let mut user_accounts = recovery_file_parse(path).unwrap();

    // DROP accounts
    let drop_file = json_path().parent().unwrap().join("drop.json");
    parse_json::drop_accounts(&mut user_accounts, &drop_file).unwrap();

    // get the supply arithmetic so that we can compare outputs
    let supply_stats = supply::populate_supply_stats_from_legacy(&user_accounts).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::mainnet(),
        &libra_genesis_default(NamedChain::MAINNET),
    )
    .unwrap();

    // NOTE: in the case of a single account being migrated, that account balance will equal the total supply as set in: SupplySettings. i.e. 10B
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx).unwrap();

    // LEAVE THIS CODE in case we need to dump this artifact.
    // test dump balances
    // compare::export_account_balances(&user_accounts, &db_rw.reader, json_path().parent().unwrap())
    //     .unwrap();

    // audit
    match compare::compare_recovery_vec_to_genesis_tx(
        &mut user_accounts,
        &db_rw.reader,
        &supply_stats,
    ) {
        Ok(list) => {
            if !list.is_empty() {
                let len = list.len();
                let out = json_path().parent().unwrap().join("audit.json");
                std::fs::write(out, serde_json::to_string_pretty(&list).unwrap()).unwrap();
                panic!("audit list is not empty, errs: {len}");
            }
        }
        Err(_e) => panic!("error creating comparison"),
    }
}
