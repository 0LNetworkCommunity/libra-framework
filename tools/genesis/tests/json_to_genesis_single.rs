//! Tests for the `make_genesis` binary.
mod support;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
// use ol_genesis_tools::compare;
use libra_genesis_tools::{
  compare,
  genesis::make_recovery_genesis_from_vec_legacy_recovery
};
use libra_types::exports::ChainId;
// use ol_types::legacy_recovery::LegacyRecovery;
use std::fs;
use support::{path_utils::json_path, test_vals};
// use libra_types::test_drop_helper::DropTemp;
use libra_framework::head_release_bundle;

#[test]
// test that a genesis blob created from struct, will actually contain the data
fn test_parse_json_for_one_validator_and_save_blob() {
    // let path = DropTemp::new_in_crate("db_rw").dir();
    let genesis_vals = test_vals::get_test_valset(4);

    let json = json_path().parent().unwrap().join("sample_end_user_single.json");

    let json_str = fs::read_to_string(json.clone()).unwrap();
    let user_accounts: Vec<LegacyRecovery> = serde_json::from_str(&json_str).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
      Some(&user_accounts),
      &genesis_vals,
      &head_release_bundle(),
      ChainId::test(),
      None
    )
    .unwrap();

    match compare::compare_recovery_vec_to_genesis_tx(&user_accounts, &gen_tx){
        Ok(list) => {
          if !list.is_empty() {
            println!("list: {:?}", &list);
            // fs::remove_file(&temp_genesis_blob_path).unwrap();
            assert!(false, "list is not empty");
          }
        },
        Err(_e) => assert!(false, "error creating comparison"),
    }

    // let vals_list = genesis_vals.into_iter().map(|v| v.address).collect();
    // match compare::check_val_set(vals_list, temp_genesis_blob_path.clone()){
    //     Ok(_) => {},
    //     Err(_) => {
    //       assert!(false, "validator set not correct");
    //       fs::remove_file(&temp_genesis_blob_path).unwrap()
    //     },
    // }

}