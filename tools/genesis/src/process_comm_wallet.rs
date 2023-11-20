use std::collections::BTreeMap;

use diem_types::account_address::AccountAddress;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
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
    recovery: &[LegacyRecovery],
    // split_factor: f64,
) -> anyhow::Result<(DonorReceipts, AllCommWallets)> {
    let mut dr = rebuild_donor_receipts(recovery)?;
    let mut cw = rebuild_cw_cumu_deposits(recovery)?;
    update_cw_with_donor(&mut cw, &mut dr);

    Ok((dr, cw))
}

/// process donor receipts
pub fn rebuild_donor_receipts(
    recovery: &[LegacyRecovery],
    // split_factor: f64,
) -> anyhow::Result<DonorReceipts> {
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
            let mut cast_receipts = ReceiptsResourceV7 {
                destination: destinations_cast,
                cumulative: temp_receipts.clone().cumulative,
                last_payment_timestamp: temp_receipts.clone().last_payment_timestamp,
                last_payment_value: temp_receipts.clone().last_payment_value,
                audit_not_found: vec![],
            };

            // // iterate through the list of payments and split
            // // then with the new split value reduce/fold into the total
            // // user payments.
            // // let user_cumu =
            // cast_receipts.cumulative.iter_mut().for_each(|this_cumu| {
            //     // mutate it
            //     *this_cumu = (split_factor * (*this_cumu as f64)) as u64;
            //     // return to next step in iter
            //     // this_cumu
            // });
            // //     .fold(0u64, |sum,  next| {
            // //          sum.checked_add(*next).expect("overflow summing cumu payments after split applied")
            // //     });

            // // // add to totals for comparison purposes
            // // total_cumu += user_cumu;

            // // same for the last_payment. Just no need to fold
            // cast_receipts.last_payment_value.iter_mut().for_each(|el| {
            //     *el = (split_factor * (*el as f64)) as u64;
            // });

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

pub fn rebuild_cw_cumu_deposits(
    recovery: &[LegacyRecovery],
    // split_factor: f64,
) -> anyhow::Result<AllCommWallets> {
    let mut total_cumu = 0;
    let mut list = BTreeMap::new();

    recovery
        .iter()
        .filter(|e| e.cumulative_deposits.is_some())
        .for_each(|e| {
            let cd = e.cumulative_deposits.as_ref().expect("no receipts field");
            // let split_value = split_factor * (cd.value as f64);
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
pub fn update_cw_with_donor(
    cw: &mut AllCommWallets,
    donors: &mut DonorReceipts,
    // split_factor: f64,
) {
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


    // TODO:


    let t = rebuild_cw_cumu_deposits(&recovery).unwrap();

    assert!(t.total_deposits == 1208569282086623, "cumu not equal");

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
        assert!(
            t.cumulative[0] == 6555272577,
            "cumu does not match"
        );
    }
}

#[test]
fn test_update_cw_from_receipts() {
    use crate::parse_json;
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    let recovery = parse_json::recovery_file_parse(p.clone()).unwrap();

    // first, test with no split
    // let split_factor = 1.0;

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

    // now add the split
    let split_factor = 2.0;

    let (_dr, cw) = prepare_cw_and_receipts(&recovery).unwrap();

    let v = cw
        .list
        .get(&AccountAddress::from_hex_literal("0x7209c13e1253ad8fb2d96a30552052aa").unwrap())
        .unwrap();

    assert!(
        v.cumulative_value == (original_value as f64 * split_factor) as u64,
        "cumu value not equal"
    );

    // assert!(
    //     v.audit_deposits_with_receipts == 116726512,
    //     "receipts value not equal"
    // );
}
