//! Tests for the `make_genesis` binary.
mod support;
use diem_state_view::account_with_state_view::AsAccountWithStateView;
use diem_storage_interface::state_view::LatestDbStateCheckpointView;
use diem_types::{account_view::AccountView, chain_id::NamedChain};
use libra_backwards_compatibility::legacy_recovery_v6::LegacyRecoveryV6;
use libra_framework::head_release_bundle;
use libra_genesis_tools::{
    compare, genesis::make_recovery_genesis_from_vec_legacy_recovery, genesis_reader, parse_json,
    supply, vm::libra_genesis_default,
};
use libra_types::{
    exports::{AccountAddress, ChainId},
    move_resource::ancestry::AncestryResource,
};

use support::{path_utils::json_path, test_vals};
#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_correct_supply_arithmetic_single() {
    let num_vals = 1;
    let genesis_vals = test_vals::get_test_valset(num_vals);

    let json = json_path().parent().unwrap().join("single.json");

    let mut user_accounts: Vec<LegacyRecoveryV6> = parse_json::recovery_file_parse(json).unwrap();

    // get the supply arithmetic so that we can compare outputs
    let supply = supply::populate_supply_stats_from_legacy(&user_accounts).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::testnet(),
        &libra_genesis_default(NamedChain::TESTNET),
    )
    .unwrap();

    // NOTE: in the case of a single account being migrated, that account balance will equal the total supply as set in: SupplySettings. i.e. 10B
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx).unwrap();
    match compare::compare_recovery_vec_to_genesis_tx(&mut user_accounts, &db_rw.reader, &supply) {
        Ok(list) => {
            if !list.is_empty() {
                panic!("list is not empty: {list:#?}");
            }
        }
        Err(_e) => panic!("error creating comparison"),
    }

    // TODO: double check supply
    compare::check_supply(4560101774012, &db_rw.reader).unwrap();
}

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_check_genesis_validators() {
    let genesis_vals = test_vals::get_test_valset(4);

    let json = json_path().parent().unwrap().join("single.json");

    let mut user_accounts: Vec<LegacyRecoveryV6> = parse_json::recovery_file_parse(json).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::test(),
        &libra_genesis_default(NamedChain::TESTING),
    )
    .unwrap();

    let vals_list: Vec<AccountAddress> =
        genesis_vals.into_iter().map(|v| v.owner_address).collect();

    compare::check_val_set(&vals_list, &gen_tx).unwrap();

    match compare::check_val_set(&vals_list, &gen_tx) {
        Ok(_) => {}
        Err(_) => {
            panic!("validator set not correct");
        }
    }
}

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_check_ancestry() {
    let genesis_vals = test_vals::get_test_valset(1);

    let json = json_path().parent().unwrap().join("single.json");

    let mut user_accounts: Vec<LegacyRecoveryV6> = parse_json::recovery_file_parse(json).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::test(),
        &libra_genesis_default(NamedChain::TESTING),
    )
    .unwrap();

    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx).unwrap();
    let db_state_view = db_rw.reader.latest_state_checkpoint_view().unwrap();

    let acc = AccountAddress::from_hex_literal(
        "0x0000000000000000000000000000000045558bad546e6159020871f7e5d094d7",
    )
    .unwrap();
    let acc_state = db_state_view.as_account_with_state_view(&acc);
    let ancestry = acc_state
        .get_resource::<AncestryResource>()
        .unwrap()
        .unwrap();
    assert_eq!(ancestry.tree.len(), 4);

    assert!(ancestry
        .tree
        .first()
        .unwrap()
        .to_string()
        .contains("46a7a744b"));
}

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_drop_single() {
    let genesis_vals = test_vals::get_test_valset(1);

    let json = json_path().parent().unwrap().join("single_dropped.json");

    let mut user_accounts: Vec<LegacyRecoveryV6> = parse_json::recovery_file_parse(json).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::test(),
        &libra_genesis_default(NamedChain::TESTING),
    )
    .unwrap();

    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx).unwrap();
    let db_state_view = db_rw.reader.latest_state_checkpoint_view().unwrap();

    let acc = AccountAddress::from_hex_literal(
        "0x0000000000000000000000000000000045558bad546e6159020871f7e5d094d7",
    )
    .unwrap();
    let acc_state = db_state_view.as_account_with_state_view(&acc);

    assert!(
        acc_state.get_account_resource().unwrap().is_none(),
        "account should not exist"
    );
}
