//! Tests for the `make_genesis` binary.
mod support;
use diem_state_view::account_with_state_view::AsAccountWithStateView;
use diem_storage_interface::state_view::LatestDbStateCheckpointView;
use diem_types::{account_view::AccountView, chain_id::NamedChain};
use libra_backwards_compatibility::legacy_recovery_v6::AccountRole;
use libra_framework::testing_local_release_bundle;
use libra_genesis_tools::{
    compare,
    genesis::make_recovery_genesis_from_vec_legacy_recovery,
    genesis_reader, parse_json,
    parse_json::recovery_file_parse,
    supply::{self},
    vm::libra_genesis_default,
};
use libra_types::{exports::ChainId, move_resource::gas_coin::GasCoinStoreResource};
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
        &testing_local_release_bundle(),
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
    use libra_types::exports::AccountAddress;
    let genesis_vals = test_vals::get_test_valset(4);

    let path = json_path()
        .parent()
        .unwrap()
        .join("sample_export_recovery.json");

    let mut user_accounts = recovery_file_parse(path).unwrap();

    // DROP accounts
    let drop_file = json_path().parent().unwrap().join("drop.json");
    parse_json::drop_accounts(&mut user_accounts, &drop_file).unwrap();

    let dead = {
        user_accounts
            .iter()
            .find(|e| {
                e.account
                    == Some(
                        AccountAddress::from_hex_literal("0x0012DD85AA97606DD22B3C9A85585D49")
                            .unwrap(),
                    )
            })
            .expect("should have this account")
            .clone()
    };

    assert!(dead.role == AccountRole::Drop);

    // // get the supply arithmetic so that we can compare outputs
    // let supply_stats = supply::populate_supply_stats_from_legacy(&user_accounts).unwrap();

    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &testing_local_release_bundle(),
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
    let dead_acct = dead.account.unwrap();
    // Ok now let's compare to what's on chain
    let db_state_view = db_rw.reader.latest_state_checkpoint_view().unwrap();
    let account_state_view = db_state_view.as_account_with_state_view(&dead_acct);

    let on_chain_balance = account_state_view
        .get_move_resource::<GasCoinStoreResource>()
        .expect("should have move resource");

    // there will be no balance struct on this account.
    assert!(on_chain_balance.is_none());
}
