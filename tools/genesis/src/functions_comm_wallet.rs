use std::collections::HashMap;

use diem_types::account_address::AccountAddress;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;
use serde::Serialize;

pub struct AllCommWallets {
    pub list: HashMap<AccountAddress, WalletState>,
    pub total_cumu: u64,
}
#[derive(Debug, Serialize)]
pub struct WalletState {
    pub cumulative_value: u64,
    pub cumulative_index: u64,
    pub depositors: Vec<AccountAddress>,
    pub audit_cumu_value: u64,
    pub audit_passes: bool,
}

#[derive(Debug)]
pub struct DonorReceipts {
  pub list: HashMap<AccountAddress, ReceiptsResourceV7>,
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

pub fn rebuild_donor_receipts(recovery: &[LegacyRecovery]) -> anyhow::Result<DonorReceipts> {
    let mut total_cumu = 0;
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
                audit_not_found: vec![]
            };

            let user_cumu = receipts.clone().cumulative.iter()
            .fold(0u64, |sum, e| { return sum.checked_add(*e).unwrap()  });

            total_cumu += user_cumu;

            let user: AccountAddress = e
                .account
                .expect("no legacy_account")
                .try_into()
                .expect("could not cast LegacyAddress");

            list.insert(user, cast_receipts);
        });

    Ok(DonorReceipts{list, total_cumu, audit_not_cw: vec![]})
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
                audit_cumu_value: 0,
                audit_passes: false,

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


pub fn update_cw_with_donor(cw: &mut AllCommWallets, donors: &mut  DonorReceipts) {
  donors.list.iter_mut().for_each(|(donor, receipt)| {
    receipt.audit_not_found = receipt.destination.iter().enumerate()
    .filter_map(|(i, &maybe_cw)| {

        if let Some(w) = cw.list.get_mut(&maybe_cw) {
            // get the cumulative value from the cumu Vec.
            w.audit_cumu_value += receipt.cumulative.get(i).expect("cant parse value");

            // populate the list of depositors to that CW
            if !w.depositors.contains(donor) { w.depositors.push(donor.clone())}
          } else  { // does this community wallet exist
          // say we can't find it in cw list
          if !donors.audit_not_cw.contains(&maybe_cw) {donors.audit_not_cw.push(maybe_cw.clone()) }

          return Some(maybe_cw)
        }
        None
    }).collect();


  });

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

    dbg!(&t.list.get(&test_addr));
    dbg!(&t.total_cumu);

}


#[test]
fn test_audit() {
    use crate::parse_json;
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    let recovery = parse_json::recovery_file_parse(p.clone()).unwrap();

    let mut dr = rebuild_donor_receipts(&recovery).unwrap();
    let mut cw = rebuild_cw_cumu_deposits(&recovery).unwrap();
    dbg!(&cw.list.values().len());

    update_cw_with_donor(&mut cw, &mut dr);

    dbg!(&dr.audit_not_cw.len());

    // let key = cw.list.keys().nth(1).unwrap();
    // dbg!(cw.list.get(key));

    std::fs::write(p.parent().unwrap().join("CW.json"), serde_json::to_string(&cw.list).unwrap()).unwrap();


}
