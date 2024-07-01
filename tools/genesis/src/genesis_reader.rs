//! helpers for reading state from a genesis blob
#![allow(clippy::mutable_key_type)] // TODO: don't quite know how to fix that warning

use anyhow::{self, bail, Context};
use diem_db::DiemDB;
use diem_executor::db_bootstrapper::{generate_waypoint, maybe_bootstrap};
use diem_state_view::account_with_state_view::AsAccountWithStateView;
use diem_storage_interface::{state_view::LatestDbStateCheckpointView, DbReader, DbReaderWriter};
use diem_temppath::TempPath;
use diem_types::{
    access_path::AccessPath,
    account_state::AccountState,
    account_view::AccountView,
    state_store::{state_key::StateKey, state_key_prefix::StateKeyPrefix},
    transaction::Transaction,
};
use diem_vm::DiemVM;
use indicatif::ProgressBar;
use libra_types::{
    exports::{AccountAddress, Waypoint},
    move_resource::coin_info::GasCoinInfoResource,
    ol_progress::OLProgress,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, CORE_CODE_ADDRESS},
};
use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

/// Compute the ledger given a genesis writeset transaction and return access to that ledger and
/// the waypoint for that state.
pub fn bootstrap_db_reader_from_gen_tx(
    genesis_transaction: &Transaction,
    // db_path: &Path,
) -> anyhow::Result<(DbReaderWriter, Waypoint)> {
    let pb = ProgressBar::new(1000)
        .with_style(OLProgress::spinner())
        .with_message("check genesis bootstraps db");
    pb.enable_steady_tick(core::time::Duration::from_millis(500));
    // iterate over the recovery file and compare balances

    let tmp_dir = TempPath::new();
    let db_rw = DbReaderWriter::new(DiemDB::new_for_test(&tmp_dir));

    assert!(db_rw
        .reader
        .get_latest_ledger_info_option()
        .unwrap()
        .is_none());

    // Bootstrap an empty DB with the genesis tx, so it has state
    let waypoint =
        generate_waypoint::<DiemVM>(&db_rw, genesis_transaction).expect("Should not fail.");
    maybe_bootstrap::<DiemVM>(&db_rw, genesis_transaction, waypoint).unwrap();

    pb.finish_and_clear();
    Ok((db_rw, waypoint))
}

/// Reads a genesis blob file and converts it into a transaction.
pub fn read_blob_to_tx(genesis_path: PathBuf) -> anyhow::Result<Transaction> {
    let mut file = File::open(genesis_path).context("unable to find genesis file")?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)
        .context("unable to read file")?;
    bcs::from_bytes(&buffer).context("unable load bytes")
}

pub const MAX_REQUEST_LIMIT: u64 = 10000;

/// Retrieves the account state for a given account address from the database.
/// Limits the number of state items to `MAX_REQUEST_LIMIT`.
pub fn get_account_state(
    db: &Arc<dyn DbReader>,
    account: AccountAddress,
    state_key_opt: Option<&StateKey>,
    // version: Version,
) -> anyhow::Result<Option<AccountState>> {
    let key_prefix = StateKeyPrefix::from(account);
    let version = db.get_latest_version()?;
    let mut iter = db.get_prefixed_state_value_iterator(&key_prefix, state_key_opt, version)?;
    let kvs = iter
        .by_ref()
        .take(MAX_REQUEST_LIMIT as usize)
        .collect::<anyhow::Result<_>>()?;
    if iter.next().is_some() {
        bail!(
            "Too many state items under state key prefix {:?}.",
            key_prefix
        );
    }
    AccountState::from_access_paths_and_values(account, &kvs)
    // todo!()
}

/// Creates a `StructTag` without type parameters for a given module and name.
fn make_struct_tag_no_types(module: &str, name: &str) -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new(module).unwrap(),
        name: Identifier::new(name).unwrap(),
        type_params: vec![],
    }
}

/// Constructs an `AccessPath` for a given account, module, and resource name.
pub fn make_access_path(
    account: AccountAddress,
    module: &str,
    name: &str,
) -> anyhow::Result<AccessPath> {
    let tag = make_struct_tag_no_types(module, name);
    AccessPath::resource_access_path(account, tag)
}

/// Computes the total supply of the gas coin by reading the state from the database.
pub fn total_supply(db_reader: &Arc<dyn DbReader>) -> Option<u128> {
    let db_state_view = db_reader.latest_state_checkpoint_view().unwrap();
    let root_account_state_view = db_state_view.as_account_with_state_view(&CORE_CODE_ADDRESS);

    let coin_info = root_account_state_view
        .get_move_resource::<GasCoinInfoResource>()
        .expect("should have move resource")
        .expect("root should have a GasCoinInfoResourcee");

    let version = db_reader.get_latest_version().unwrap();

    coin_info
        .supply()
        .as_ref()
        .map(|o| match o.aggregator.as_ref() {
            Some(aggregator) => {
                let state_key = aggregator.state_key();
                let value = db_reader
                    .get_state_value_by_version(&state_key, version)
                    .expect("aggregator value must exist in data store")
                    .expect("supply value exists");
                // todo!()
                bcs::from_bytes(value.bytes()).unwrap()
            }
            None => o.integer.as_ref().unwrap().value,
        })
}

/// Tests the database reader-writer initialization and state value retrieval.
#[test]
fn test_db_rw() {
    use diem_db::DiemDB;
    use diem_executor::db_bootstrapper::maybe_bootstrap;
    use diem_temppath::TempPath;
    use diem_types::state_store::state_key::StateKey;
    use libra_types::exports::AccountAddress;

    let tmp_dir = TempPath::new().path().to_owned();

    let temp_db = DiemDB::new_for_test(&tmp_dir);
    let db_rw = DbReaderWriter::new(temp_db);

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/genesis.blob");
    let genesis_txn = read_blob_to_tx(p).unwrap();

    // Bootstrap empty DB.
    let waypoint = generate_waypoint::<DiemVM>(&db_rw, &genesis_txn).expect("Should not fail.");
    maybe_bootstrap::<DiemVM>(&db_rw, &genesis_txn, waypoint).unwrap();

    let ap = make_access_path(AccountAddress::ZERO, "slow_wallet", "SlowWalletList").unwrap();
    let version = db_rw.reader.get_latest_version().unwrap();
    let _bytes = db_rw
        .reader
        .get_state_value_by_version(&StateKey::access_path(ap), version)
        .unwrap();
}
