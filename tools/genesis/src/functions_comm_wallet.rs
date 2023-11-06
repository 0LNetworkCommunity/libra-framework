use std::collections::HashMap;

use diem_types::account_address::AccountAddress;
use libra_types::legacy_types::legacy_recovery::LegacyRecovery;

pub struct AllCommWallets {
    pub list: HashMap<AccountAddress, WalletState>,
}
pub struct WalletState {
    pub cumulative_value: u64,
    pub cumulative_index: u64,
    pub depositors: Vec<AccountAddress>,
}

pub struct DonorReceipts {
    pub list: HashMap<AccountAddress, ReceiptsResourceV7>,
}

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
            let destinations_cast: Vec<AccountAddress> = receipts.destination
                .iter()
                .map(|a| a.try_into().expect("could not cast LegacyAdresss"))
                .collect();
            let cast_receipts = &ReceiptsResourceV7 {
                destination: destinations_cast,
                cumulative: receipts.cumulative,
                last_payment_timestamp: receipts.last_payment_timestamp,
                last_payment_value: receipts.last_payment_value,
            };

            let user = e.account
            .expect("could not get account addr")
            .try_into()
            .expect("could not cast LegacyAddress");

            list.insert(user, cast_receipts.to_owned());
        });

    Ok(DonorReceipts { list })
}

pub fn get_cw_cumu_deposits(recovery: &[LegacyRecovery]) -> anyhow::Result<AllCommWallets> {
    // filter recovery for ones marked community wallet, and return cumulative
    // deposits
    // push to hashmap

    todo!()
}
