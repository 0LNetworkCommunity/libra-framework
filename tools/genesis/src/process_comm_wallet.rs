use diem_types::account_address::AccountAddress;
use libra_backwards_compatibility::legacy_recovery_v6::LegacyRecoveryV6;
use serde::Serialize;
use std::collections::BTreeMap;

// Represents the state of all community wallets
pub struct AllCommWallets {
    pub list: BTreeMap<AccountAddress, WalletState>,
    pub total_deposits: u64,
}

// Represents the state of an individual wallet
#[derive(Debug, Serialize)]
pub struct WalletState {
    pub cumulative_value: u64,
    pub cumulative_index: u64,
    pub depositors: Vec<AccountAddress>,
    pub audit_deposits_with_receipts: u64,
}

// Represents donor receipts, including a list of receipts, total cumulative value,
// and addresses not found in community wallets.
#[derive(Debug)]
pub struct DonorReceipts {
    pub list: BTreeMap<AccountAddress, ReceiptsResourceV7>,
    pub total_cumu: u64,
    pub audit_not_cw: Vec<AccountAddress>,
}

// Represents a resource containing receipt information for a donor.
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
    recovery: &[LegacyRecoveryV6],
    // split_factor: f64,
) -> anyhow::Result<(DonorReceipts, AllCommWallets)> {
    let mut dr = rebuild_donor_receipts(recovery)?;
    let mut cw = rebuild_cw_cumu_deposits(recovery)?;
    // update the donor list in cw
    update_cw_with_donor(&mut cw, &mut dr);

    Ok((dr, cw))
}

/// process donor receipts
pub fn rebuild_donor_receipts(recovery: &[LegacyRecoveryV6]) -> anyhow::Result<DonorReceipts> {
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

            // this resource should now show the split numbers
            let cast_receipts = ReceiptsResourceV7 {
                destination: temp_receipts.clone().destination,
                cumulative: temp_receipts.clone().cumulative,
                last_payment_timestamp: temp_receipts.clone().last_payment_timestamp,
                last_payment_value: temp_receipts.clone().last_payment_value,
                audit_not_found: vec![],
            };

            list.insert(e.account.unwrap(), cast_receipts);
        });

    Ok(DonorReceipts {
        list,
        total_cumu,
        audit_not_cw: vec![],
    })
}

/// Processes cumulative deposits for community wallets.
pub fn rebuild_cw_cumu_deposits(recovery: &[LegacyRecoveryV6]) -> anyhow::Result<AllCommWallets> {
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
            total_cumu += cd.value;

            let cast_receipts = WalletState {
                cumulative_value: cd.value,
                cumulative_index: cd.index,
                depositors: vec![],
                audit_deposits_with_receipts: 0,
            };

            list.insert(e.account.unwrap(), cast_receipts);
        });

    Ok(AllCommWallets {
        list,
        total_deposits: total_cumu,
    })
}

/// extract donor addresses from receipts and place into new
/// community wallet struct
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

    // scaled
    let t = rebuild_cw_cumu_deposits(&recovery).unwrap();

    assert_eq!(t.total_deposits, 52488307886371112, "cumu not equal");
    assert_eq!(t.list.len(), 145, "len not equal");
}

#[test]
fn test_receipt_recovery() {
    use crate::parse_json;

    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let recovery = parse_json::recovery_file_parse(p).unwrap();

    let t = rebuild_donor_receipts(&recovery).unwrap();
    let test_addr = "00000000000000000000000000000000123c6ca26a6ed35ad00868b33b4a98d1"
        .parse::<AccountAddress>()
        .unwrap();

    if let Some(t) = t.list.get(&test_addr) {
        assert_eq!(t.cumulative[0], 231401421509, "cumu does not match");
    }
}

#[test]
fn test_update_cw_from_receipts() {
    use crate::parse_json;
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let recovery = parse_json::recovery_file_parse(p).unwrap();

    let (_dr, cw) = prepare_cw_and_receipts(&recovery).unwrap();

    let v = cw
        .list
        .get(&AccountAddress::from_hex_literal("0x7209c13e1253ad8fb2d96a30552052aa").unwrap())
        .unwrap();

    assert_eq!(v.cumulative_value, 6405927426, "cumu value not equal");
}
