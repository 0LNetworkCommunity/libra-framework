use std::path::Path;

use anyhow::Result;
use diem_types::account_view::AccountView;
use libra_backwards_compatibility::version_five::state_snapshot_v5::v5_accounts_from_manifest_path;
use libra_storage::read_snapshot::{accounts_from_snapshot_backup, load_snapshot_manifest};
use libra_types::exports::AccountAddress;

use crate::table_structs::{WarehouseAccount, WarehouseRecord, WarehouseTime};

// uses libra-compatibility to parse the v5 manifest files, and decode v5 format bytecode into current version data structures (v6+);
pub async fn extract_v5_snapshot(v5_manifest_path: &Path) -> Result<Vec<WarehouseRecord>> {
    let account_blobs = v5_accounts_from_manifest_path(v5_manifest_path).await?;
    dbg!(&account_blobs.len());
    let mut warehouse_state = vec![];
    for el in account_blobs.iter() {
        let acc = el.to_account_state()?;
        // convert v5 address to v7
        match acc.get_address() {
            Ok(a) => {
                let address_literal = a.to_hex_literal();
                let cast_address = AccountAddress::from_hex_literal(&address_literal)?;
                let s = WarehouseRecord {
                    account: WarehouseAccount {
                        address: cast_address,
                    },
                    time: WarehouseTime::default(),
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

pub async fn extract_current_snapshot(archive_path: &Path) -> Result<Vec<WarehouseRecord>> {
    let manifest_file = archive_path.join("state.manifest");
    assert!(
        manifest_file.exists(),
        "{}",
        &format!("state.manifest file not found at {:?}", archive_path)
    );
    let manifest = load_snapshot_manifest(&manifest_file)?;

    let accs = accounts_from_snapshot_backup(manifest, archive_path).await?;

    // TODO: Change to log
    println!("SUCCESS: backup loaded. # accounts: {}", &accs.len());

    // TODO: stream this
    let mut warehouse_state = vec![];
    for el in accs.iter() {
        if let Some(address) = el.get_account_address()? {
            let s = WarehouseRecord {
                account: WarehouseAccount { address },
                time: WarehouseTime::default(),
                balance: None,
            };
            warehouse_state.push(s);
        }
    }

    // TODO: Change to log
    println!(
        "SUCCESS: accounts parsed. # accounts: {}",
        &warehouse_state.len()
    );

    if warehouse_state.len() != accs.len() {
        println!("WARN: account count does not match");
    }

    Ok(warehouse_state)
}
