use std::collections::HashMap;

use diem_types::account_address::AccountAddress;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;

pub struct AllCommWallets {
    pub list: HashMap<AccountAddress, WalletState>,
    pub total_cumu: u64,
}
pub struct WalletState {
    pub cumulative_value: u64,
    pub cumulative_index: u64,
    pub depositors: Vec<AccountAddress>,
}

#[derive(Debug)]
pub struct DonorReceipts(HashMap<AccountAddress, ReceiptsResourceV7>);

#[derive(Debug)]
pub struct ReceiptsResourceV7 {
    pub destination: Vec<AccountAddress>,
    pub cumulative: Vec<u64>,
    pub last_payment_timestamp: Vec<u64>,
    pub last_payment_value: Vec<u64>,
}

pub fn rebuild_donor_receipts(recovery: &[LegacyRecovery]) -> anyhow::Result<DonorReceipts> {
    let mut list = HashMap::new();

    recovery
        .iter()
        .filter(|e| e.receipts.is_some())
        .for_each(|e| {
            let receipts = e.receipts.as_ref().expect("no receipts field");
            let destinations_cast: Vec<AccountAddress> = receipts
                .destination
                .iter()
                .map(|&a| a.try_into().expect("could not cast LegacyAdresss"))
                .collect();
            let cast_receipts = ReceiptsResourceV7 {
                destination: destinations_cast,
                cumulative: receipts.clone().cumulative,
                last_payment_timestamp: receipts.clone().last_payment_timestamp,
                last_payment_value: receipts.clone().last_payment_value,
            };

            let user: AccountAddress = e
                .account
                .expect("no legacy_account")
                .try_into()
                .expect("could not cast LegacyAddress");

            list.insert(user, cast_receipts);
        });

    Ok(DonorReceipts(list))
}

pub fn rebuild_cw_cumu_deposits(recovery: &[LegacyRecovery]) -> anyhow::Result<AllCommWallets> {
    let mut total_cumu = 0;
    let mut list = HashMap::new();

    recovery
        .iter()
        .filter(|e| e.cumulative_deposits.is_some())
        .for_each(|e| {
            let cd = e.cumulative_deposits.as_ref().expect("no receipts field");
            total_cumu += cd.value;

            let cast_receipts = WalletState {
                cumulative_value: cd.value.clone(),
                cumulative_index: cd.index.clone(),
                depositors: vec![],
            };

            let user: AccountAddress = e
                .account
                .expect("could not get account addr")
                .try_into()
                .expect("could not cast LegacyAddress");

            list.insert(user, cast_receipts);
        });

    Ok(AllCommWallets { list, total_cumu })
}

#[test]
fn test_cw_recovery() {
    use crate::parse_json;

    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");

    let recovery = parse_json::recovery_file_parse(p).unwrap();

    let t = rebuild_cw_cumu_deposits(&recovery).unwrap();

    dbg!(&t.total_cumu);
    assert!(t.total_cumu == 1208569282086623,"not equal");
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

    dbg!(&t.0.get(&test_addr));
}
