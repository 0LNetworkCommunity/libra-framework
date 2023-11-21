//! Tests for the `make_genesis` binary.
mod support;
use diem_types::chain_id::NamedChain;
use libra_framework::head_release_bundle;
use libra_genesis_tools::genesis_reader;
use libra_genesis_tools::parse_json::recovery_file_parse;
use libra_genesis_tools::supply::{self, SupplySettings};
use libra_genesis_tools::vm::libra_genesis_default;
use libra_genesis_tools::{compare, genesis::make_recovery_genesis_from_vec_legacy_recovery};
use libra_types::exports::ChainId;
use libra_types::legacy_types::legacy_address::LegacyAddress;
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
    let map_dd_to_slow = vec![
        // FTW
        "3A6C51A0B786D644590E8A21591FA8E2"
            .parse::<LegacyAddress>()
            .unwrap(),
        // tip jar
        "2B0E8325DEA5BE93D856CFDE2D0CBA12"
            .parse::<LegacyAddress>()
            .unwrap(),
    ];
    // get the supply arithmetic so that we can compare outputs
    let mut supply_stats =
        supply::populate_supply_stats_from_legacy(&user_accounts, &map_dd_to_slow).unwrap();
    let supply_settings = SupplySettings {
        target_supply: 100_000_000_000.0,
        target_future_uses: 0.70,
        years_escrow: 7,
        map_dd_to_slow,
    };
    supply_stats
        .set_ratios_from_settings(&supply_settings)
        .unwrap();

    dbg!(&supply_stats);
    let gen_tx = make_recovery_genesis_from_vec_legacy_recovery(
        &mut user_accounts,
        &genesis_vals,
        &head_release_bundle(),
        ChainId::mainnet(),
        Some(supply_settings),
        &libra_genesis_default(NamedChain::MAINNET),
    )
    .unwrap();

    // NOTE: in the case of a single account being migrated, that account balance will equal the total supply as set in: SupplySettings. i.e. 10B
    let (db_rw, _) = genesis_reader::bootstrap_db_reader_from_gen_tx(&gen_tx).unwrap();
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
