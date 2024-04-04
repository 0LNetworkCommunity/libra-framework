use std::collections::BTreeMap;

use diem_types::account_address::AccountAddress;
use libra_types::legacy_types::legacy_recovery_v5::LegacyRecoveryV5;
use serde::Serialize;

pub struct AllCommWallets {
    pub list: BTreeMap<AccountAddress, WalletState>,
    pub total_deposits: u64,
}
#[derive(Debug, Serialize)]
pub struct WalletState {
    pub cumulative_value: u64,
    pub cumulative_index: u64,
    pub depositors: Vec<AccountAddress>,
    pub audit_deposits_with_receipts: u64,
}

#[derive(Debug)]
pub struct DonorReceipts {
    pub list: BTreeMap<AccountAddress, ReceiptsResourceV7>,
    pub total_cumu: u64,
    pub audit_not_cw: Vec<AccountAddress>,
}

#[derive(Debug)]
pub struct ReceiptsResourceV7 {
    pub destination: Vec<AccountAddress>,
    pub cumulative: Vec<u64>,
    pub last_payment_timestamp: Vec<u64>,
    pub last_payment_value: Vec<u64>,
    pub audit_not_found: Vec<AccountAddress>,
}

/// do the entire workflow of processing community wallet accounts
/// and inserting the donor information based on receipts
pub fn prepare_cw_and_receipts(
    recovery: &[LegacyRecoveryV5],
    // split_factor: f64,
) -> anyhow::Result<(DonorReceipts, AllCommWallets)> {
    let mut dr = rebuild_donor_receipts(recovery)?;
    let mut cw = rebuild_cw_cumu_deposits(recovery)?;
    // update the donor list in cw
    update_cw_with_donor(&mut cw, &mut dr);

    Ok((dr, cw))
}

/// process donor receipts
pub fn rebuild_donor_receipts(recovery: &[LegacyRecoveryV5]) -> anyhow::Result<DonorReceipts> {
    let total_cumu = 0;
    let mut list = BTreeMap::new();

    recovery
        .iter()
        .filter(|e| e.receipts.is_some())
        .for_each(|e| {
            // NOTE this is not mutable because we don't want to change
            // the underlying LegacyRecovery. So it will be intentionally
            // less efficient
            let temp_receipts = e.receipts.as_ref().expect("no receipts field");
            let destinations_cast: Vec<AccountAddress> = temp_receipts
                .destination
                .iter()
                .map(|&a| a.try_into().expect("could not cast LegacyAdresss"))
                .collect();
            // this resource should now show the split numbers
            let cast_receipts = ReceiptsResourceV7 {
                destination: destinations_cast,
                cumulative: temp_receipts.clone().cumulative,
                last_payment_timestamp: temp_receipts.clone().last_payment_timestamp,
                last_payment_value: temp_receipts.clone().last_payment_value,
                audit_not_found: vec![],
            };

            let user: AccountAddress = e
                .account
                .expect("no legacy_account")
                .try_into()
                .expect("could not cast LegacyAddress");

            list.insert(user, cast_receipts);
        });

    Ok(DonorReceipts {
        list,
        total_cumu,
        audit_not_cw: vec![],
    })
}

pub fn rebuild_cw_cumu_deposits(recovery: &[LegacyRecoveryV5]) -> anyhow::Result<AllCommWallets> {
    let mut total_cumu = 0;
    let mut list = BTreeMap::new();

    recovery
        .iter()
        .filter(|e| e.cumulative_deposits.is_some())
        .for_each(|e| {
            let cd = e
                .cumulative_deposits
                .as_ref()
                .expect("no cumulative deposits field");
            // dbg!(&cd.value);
            total_cumu += cd.value;

            let cast_receipts = WalletState {
                cumulative_value: cd.value,
                cumulative_index: cd.index,
                depositors: vec![],
                audit_deposits_with_receipts: 0,
            };

            let user: AccountAddress = e
                .account
                .expect("could not get account addr")
                .try_into()
                .expect("could not cast LegacyAddress");

            list.insert(user, cast_receipts);
        });

    Ok(AllCommWallets {
        list,
        total_deposits: total_cumu,
    })
}

/// extract donor addresses from receipts and place into new
/// communit wallet struct
pub fn update_cw_with_donor(cw: &mut AllCommWallets, donors: &mut DonorReceipts) {
    donors.list.iter_mut().for_each(|(donor, receipt)| {
        receipt.audit_not_found = receipt
            .destination
            .iter()
            .enumerate()
            .filter_map(|(i, &maybe_cw)| {
                if let Some(w) = cw.list.get_mut(&maybe_cw) {
                    // get the cumulative value from the cumu Vec.

                    let value = receipt.cumulative.get(i).expect("cant parse value");
                    w.audit_deposits_with_receipts += value;

                    // populate the list of depositors to that CW
                    if !w.depositors.contains(donor) {
                        w.depositors.push(*donor)
                    }
                } else {
                    // does this community wallet exist
                    // say we can't find it in cw list
                    if !donors.audit_not_cw.contains(&maybe_cw) {
                        donors.audit_not_cw.push(maybe_cw)
                    }

                    return Some(maybe_cw);
                }
                None
            })
            .collect();
    });
}

#[test]
fn test_cw_recovery() {
    use crate::parse_json;

    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let recovery = parse_json::recovery_file_parse(p).unwrap();

    // test splitting the coin and get scale factor
    // let settings = SupplySettings {
    //     target_supply: 100_000_000_000.0, // 100B times scaling factor
    //     target_future_uses: 0.70,
    //     years_escrow: 7,
    //     map_dd_to_slow: vec![
    //         // FTW
    //         "3A6C51A0B786D644590E8A21591FA8E2"
    //             .parse::<LegacyAddress>()
    //             .unwrap(),
    //         // tip jar
    //         "2B0E8325DEA5BE93D856CFDE2D0CBA12"
    //             .parse::<LegacyAddress>()
    //             .unwrap(),
    //     ],
    // };
    // let mut supply =
    //     populate_supply_stats_from_legacy(&recovery, &settings.map_dd_to_slow).unwrap();
    // supply.set_ratios_from_settings(&settings).unwrap();
    // dbg!(&supply);

    // Check calcs with no supply scaling
    let t = rebuild_cw_cumu_deposits(&recovery).unwrap();
    assert!(t.total_deposits == 1208569282086623, "cumu not equal");

    // recovery.iter_mut().for_each(|e| {
    //     genesis_functions::util_scale_all_coins(e, &supply).unwrap();
    // });

    // scaled
    let t = rebuild_cw_cumu_deposits(&recovery).unwrap();
    assert!(t.total_deposits == 50324012162657770, "cumu not equal");

    assert!(t.list.len() == 134, "len not equal");
}

#[test]
fn test_receipt_recovery() {
    use crate::parse_json;

    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let recovery = parse_json::recovery_file_parse(p.clone()).unwrap();

    // first, test with no split
    // let split_factor = 1.0;

    let t = rebuild_donor_receipts(&recovery).unwrap();
    let test_addr = "00000000000000000000000000000000123c6ca26a6ed35ad00868b33b4a98d1"
        .parse::<AccountAddress>()
        .unwrap();

    dbg!(&t.list.get(&test_addr));

    if let Some(t) = t.list.get(&test_addr) {
        assert!(t.cumulative[0] == 6555272577, "cumu does not match");
    }

    // Do it again with a split factor
    let recovery = parse_json::recovery_file_parse(p).unwrap();
    // let split_factor = 2.0;

    let t = rebuild_donor_receipts(&recovery).unwrap();
    let test_addr = "00000000000000000000000000000000123c6ca26a6ed35ad00868b33b4a98d1"
        .parse::<AccountAddress>()
        .unwrap();

    if let Some(t) = t.list.get(&test_addr) {
        assert!(t.cumulative[0] == 6555272577, "cumu does not match");
    }
}

#[test]
fn test_update_cw_from_receipts() {
    use crate::parse_json;
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    let recovery = parse_json::recovery_file_parse(p.clone()).unwrap();

    let (_dr, cw) = prepare_cw_and_receipts(&recovery).unwrap();

    let v = cw
        .list
        .get(&AccountAddress::from_hex_literal("0x7209c13e1253ad8fb2d96a30552052aa").unwrap())
        .unwrap();

    let original_value = 162900862;
    assert!(v.cumulative_value == original_value, "cumu value not equal");
    assert!(
        v.audit_deposits_with_receipts == 116726512,
        "receipts value not equal"
    );

    let recovery = parse_json::recovery_file_parse(p).unwrap();

    let (_dr, cw) = prepare_cw_and_receipts(&recovery).unwrap();

    let v = cw
        .list
        .get(&AccountAddress::from_hex_literal("0x7209c13e1253ad8fb2d96a30552052aa").unwrap())
        .unwrap();

    assert!(v.cumulative_value == original_value, "cumu value not equal");

    // assert!(
    //     v.audit_deposits_with_receipts == 116726512,
    //     "receipts value not equal"
    // );
}
