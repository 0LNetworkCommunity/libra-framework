//! Tests for the `make_genesis` binary.
mod support;
use libra_framework::head_release_bundle;
use libra_genesis_tools::supply::{self, SupplySettings};
use libra_genesis_tools::{compare, genesis::make_recovery_genesis_from_vec_legacy_recovery};
use libra_types::exports::ChainId;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
use std::fs;
use support::{path_utils::json_path, test_vals};
use libra_types::exports::AccountAddress;

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_correct_supply_arithmetic_single() {
    // let path = DropTemp::new_in_crate("db_rw").dir();
    let num_vals = 1;
    let genesis_vals = test_vals::get_test_valset(num_vals);

    let json = json_path()
        .parent()
        .unwrap()
        .join("sample_end_user_single.json");

    let json_str = fs::read_to_string(json.clone()).unwrap();
    let user_accounts: Vec<LegacyRecovery> = serde_json::from_str(&json_str).unwrap();

    // get the supply arithmetic so that we can compare outputs
    let mut supply =
        supply::populate_supply_stats_from_legacy(&user_accounts, &vec![]).unwrap();
    let supply_settings = SupplySettings::default();
    supply
        .set_ratios_from_settings(&supply_settings)
        .unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        Some(&user_accounts),
        &genesis_vals,
        &head_release_bundle(),
        ChainId::test(),
        Some(supply_settings.clone()),
    )
    .unwrap();

    // NOTE: in the case of a single account being migrated, that account balance will equal the total supply as set in: SupplySettings. i.e. 10B

    match compare::compare_recovery_vec_to_genesis_tx(&user_accounts, &gen_tx, &supply) {
        Ok(list) => {
            if !list.is_empty() {
                assert!(false, "list is not empty: {list:#?}");
            }
        }
        Err(_e) => assert!(false, "error creating comparison"),
    }
    let expected_supply = supply_settings.scale_supply() as u64 + (num_vals * 10000000000) as u64; // the genesis reward at testnet for one validator
    compare::check_supply(expected_supply, &gen_tx).unwrap();

}


#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_check_genesis_validators() {
    // let path = DropTemp::new_in_crate("db_rw").dir();
    let genesis_vals = test_vals::get_test_valset(4);

    let json = json_path()
        .parent()
        .unwrap()
        .join("sample_end_user_single.json");

    let json_str = fs::read_to_string(json.clone()).unwrap();
    let user_accounts: Vec<LegacyRecovery> = serde_json::from_str(&json_str).unwrap();


    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        Some(&user_accounts),
        &genesis_vals,
        &head_release_bundle(),
        ChainId::test(),
        None,
    )
    .unwrap();


    let vals_list: Vec<AccountAddress> = genesis_vals.into_iter().map(|v| v.owner_address).collect();

    compare::check_val_set(&vals_list, &gen_tx).unwrap();

    match compare::check_val_set(&vals_list, &gen_tx){
        Ok(_) => {},
        Err(_) => {
          assert!(false, "validator set not correct");
        },
    }
}