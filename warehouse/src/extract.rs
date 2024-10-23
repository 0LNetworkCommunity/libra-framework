use std::path::Path;

use anyhow::Result;
use libra_backwards_compatibility::version_five::state_snapshot_v5::v5_accounts_from_manifest_path;
use libra_types::exports::AccountAddress;

use crate::table_structs::{WarehouseAccount, WarehouseState};

pub async fn extract_v5_snapshot(v5_manifest_path: &Path) -> Result<Vec<WarehouseState>> {
    let account_blobs = v5_accounts_from_manifest_path(&v5_manifest_path).await?;
    dbg!(&account_blobs.len());
    let mut warehouse_state = vec![];
    for el in account_blobs.iter() {
        let acc = el.to_account_state()?;
        // convert v5 address to v7
        match acc.get_address() {
            Ok(a) => {
                let address_literal = a.to_hex_literal();
                let cast_address = AccountAddress::from_hex_literal(&address_literal)?;
                let s = WarehouseState {
                    account: WarehouseAccount {
                        address: cast_address,
                    },
                    balance: None,
                };
                warehouse_state.push(s);
            }
            Err(e) => {
                println!("WARN: could not parse blob to V5 Address{}", &e);
            }
        }
    }

    Ok(warehouse_state)
}
