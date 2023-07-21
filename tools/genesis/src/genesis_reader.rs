//! helpers for reading state from a genesis blob

// use crate::db_utils;
use anyhow::{self, Context};
use libra_types::exports::AccountAddress;
use move_core_types::identifier::Identifier;
use zapatos_executor::db_bootstrapper::maybe_bootstrap;
// use libra_types::legacy_types::ancestry::AncestryResource;
// use libra_types::exports::AccountAddress;
// use libra_types::legacy_types::legacy_address::LegacyAddress;
// use libra_types::legacy_types::legacy_recovery::{LegacyRecovery, read_from_recovery_file};
use libra_types::exports::Waypoint;
use move_core_types::language_storage::{StructTag, CORE_CODE_ADDRESS};
use zapatos_types::access_path::AccessPath;
// use libra_types::ol_progress::OLProgress;
// use zapatos_types::access_path::AccessPath;
// use zapatos_types::state_store::state_key::StateKey;
use zapatos_types::transaction::Transaction;
// use zapatos_types::{
//   account_view::AccountView,
//   account_state::AccountState,
//   move_resource::MoveStorage,
// };
use zapatos_storage_interface::DbReaderWriter;
use zapatos_vm::AptosVM;
// use std::convert::TryFrom;
// use std::path::PathBuf;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use zapatos_executor::db_bootstrapper::generate_waypoint;
// use std::ops::Deref;
// use indicatif::{ProgressIterator, ProgressBar};

/// Compute the ledger given a genesis writeset transaction and return access to that ledger and
/// the waypoint for that state.
pub fn read_db_and_compute_genesis(
    _genesis_path: &Path,
    // db_path: &Path,
) -> anyhow::Result<(DbReaderWriter, Waypoint)> {

    // let mut file = File::open(genesis_path).context("unable to find genesis file")?;
    // let mut buffer = vec![];
    // file.read_to_end(&mut buffer).context("unable to read file")?;
    // let _genesis: Transaction = bcs::from_bytes(&buffer).context("unable load bytes")?;

    // let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis.0));
    // let tmp_dir = TempPath::new();
    // let db_rw = DbReaderWriter::new(AptosDB::new_for_test(&tmp_dir));

    // assert!(db_rw
    //     .reader
    //     .get_latest_ledger_info_option()
    //     .unwrap()
    //     .is_none());

    // // Bootstrap empty DB.
    // let waypoint = generate_waypoint::<AptosVM>(&db_rw, &genesis_txn).expect("Should not fail.");
    // maybe_bootstrap::<AptosVM>(&db_rw, &genesis_txn, waypoint).unwrap();
    // let ledger_info = db_rw.reader.get_latest_ledger_info().unwrap();

    // let diemdb = DiemDB::open(
    //     db_path,
    //     false,
    //     None,
    //     RocksdbConfig::default(),
    //     true, /* account_count_migration */
    // )
    // .map_err(|e| Error::UnexpectedError(e.to_string()))?;
    // let db_rw = DbReaderWriter::new(diemdb);

    // let mut file = File::open(genesis_path)
    //     .map_err(|e| Error::UnexpectedError(format!("Unable to open genesis file: {}", e)))?;
    // let mut buffer = vec![];
    // file.read_to_end(&mut buffer)
    //     .map_err(|e| Error::UnexpectedError(format!("Unable to read genesis: {}", e)))?;
    // let genesis = bcs::from_bytes(&buffer)
    //     .map_err(|e| Error::UnexpectedError(format!("Unable to parse genesis: {}", e)))?;

    // let waypoint = db_bootstrapper::generate_waypoint::<DiemVM>(&db_rw, &genesis)
    //     .map_err(|e| Error::UnexpectedError(e.to_string()))?;
    // db_bootstrapper::maybe_bootstrap::<DiemVM>(&db_rw, &genesis, waypoint)
    //     .map_err(|e| Error::UnexpectedError(format!("Unable to commit genesis: {}", e)))?;

    // Ok((db_rw, waypoint))
    todo!()
}


pub fn read_blob_to_tx(genesis_path: PathBuf) -> anyhow::Result<Transaction> {
    let mut file = File::open(genesis_path).context("unable to find genesis file")?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).context("unable to read file")?;
    bcs::from_bytes(&buffer).context("unable load bytes")
}

#[test]
fn test_db_rw() {
    use libra_types::test_drop_helper::DropTemp;
    use zapatos_db::AptosDB;
    // use libra_types::legacy_types::ancestry::AncestryResource;
    use libra_types::exports::AccountAddress;
    use zapatos_types::state_store::state_key::StateKey;


    let tmp_dir = DropTemp::new_in_crate("db_rw").dir();
    let db_rw = DbReaderWriter::new(AptosDB::new_for_test(&tmp_dir));

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/genesis.blob");
    let genesis_txn = read_blob_to_tx(p).unwrap();

    // Bootstrap empty DB.
    let waypoint = generate_waypoint::<AptosVM>(&db_rw, &genesis_txn).expect("Should not fail.");
    maybe_bootstrap::<AptosVM>(&db_rw, &genesis_txn, waypoint).unwrap();
    // let ledger_info = db_rw.reader.get_latest_ledger_info().unwrap();

  let ap = make_access_path(AccountAddress::ZERO, "slow_wallet", "SlowWalletList").unwrap();
  let version = db_rw.reader.get_latest_version().unwrap();
  let bytes = db_rw.reader.get_state_value_by_version(&StateKey::access_path(ap), version).unwrap();


  dbg!(&bytes);

  //   };

}


fn make_struct_tag_no_types(module: &str, name: &str) -> StructTag {
   StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new(module).unwrap(),
        name: Identifier::new(name).unwrap(),
        type_params: vec![],
    }
}

pub fn make_access_path(account: AccountAddress, module: &str, name: &str) -> anyhow::Result<AccessPath> {
    let tag = make_struct_tag_no_types(module, name);
    AccessPath::resource_access_path(account, tag)
}

// fn get_move_resource(db_rw: &DbReaderWriter, path: AccessPath) -> anyhow::Result<Vec<u8>> {
//   let version = db_rw.get_latest_version()?;
//     let state_bytes = db_rw.get_state_value_by_version(
//         &StateKey::access_path(path),
//         version,
//     )?;
//     Ok(state_bytes)
// }