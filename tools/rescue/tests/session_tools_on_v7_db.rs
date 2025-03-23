use diem_storage_interface::state_view::DbStateViewAtVersion;
use diem_storage_interface::DbReaderWriter;
use diem_temppath::TempPath;
use diem_vm::move_vm_ext::SessionId;
use flate2::read::GzDecoder;
use libra_framework::release::ReleaseTarget;
use std::path::Path;
use std::{fs, path::PathBuf};
use tar::Archive;

// Database related imports
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;

// VM related imports
use move_core_types::language_storage::CORE_CODE_ADDRESS;

// Project-specific imports
use libra_rescue::session_tools::{
    libra_run_session, upgrade_framework_changeset, writeset_voodoo_events,
};

/// Sets up a test database by extracting a fixture file to a temporary directory.
///
/// This function extracts the database fixture from `./rescue/fixtures/db_339.tar.gz`,
/// which contains a recovered database at epoch 339. The extracted database is placed
/// in a temporary directory that will be automatically cleaned up when the TempPath
/// is dropped (unless persist() is called).
///
/// Returns the TempPath containing the extracted database.
fn setup_test_db() -> anyhow::Result<PathBuf> {
    let mut temp_dir = TempPath::new();
    temp_dir.create_as_dir()?;
    temp_dir.persist();

    // Open and decompress the fixture file
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixture_path = Path::new(manifest_dir).join("fixtures/db_339.tar.gz");
    assert!(&fixture_path.exists(), "can't find fixture db_339.tar.gz");
    let tar_gz = fs::File::open(fixture_path)?;
    let decompressor = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressor);

    // Extract to temp directory
    archive.unpack(temp_dir.path())?;

    Ok(temp_dir.path().join("db_339"))
}

#[test]
/// Test we can publish a framework to a database fixture
///
/// Uses a database fixture extracted from `./rescue/fixtures/db_339.tar.gz`
fn test_publish() -> anyhow::Result<()> {
    let dir = setup_test_db()?;
    let upgrade_mrb = ReleaseTarget::Head
        .find_bundle_path()
        .expect("cannot find head.mrb");
    dbg!(&upgrade_mrb);
    upgrade_framework_changeset(&dir, None, &upgrade_mrb)?;
    Ok(())
}

#[test]
/// The writeset voodoo needs to be perfect
///
/// Uses a database fixture extracted from `./rescue/fixtures/db_339.tar.gz`
fn test_voodoo() -> anyhow::Result<()> {
    let dir = setup_test_db()?;
    libra_run_session(dir, writeset_voodoo_events, None, None)?;
    Ok(())
}

#[test]
/// Testing we can open a database from fixtures, and produce a VM session
///
/// Uses a database fixture extracted from `./rescue/fixtures/db_339.tar.gz`
fn meta_test_open_db_sync() -> anyhow::Result<()> {
    let dir = setup_test_db()?;

    let db = DiemDB::open(
        dir,
        true,
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner */
        RocksdbConfigs::default(),
        false, /* indexer */
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    )
    .expect("Failed to open DB.");

    let db_rw = DbReaderWriter::new(db);

    let v = db_rw.reader.get_latest_version().unwrap();

    let view = db_rw.reader.state_view_at_version(Some(v)).unwrap();

    let dvm = diem_vm::DiemVM::new(&view);
    let adapter = dvm.as_move_resolver(&view);

    let _s_id = SessionId::Txn {
        sender: CORE_CODE_ADDRESS,
        sequence_number: 0,
        script_hash: b"none".to_vec(),
    };
    let s_id = SessionId::genesis(diem_crypto::HashValue::zero());

    let mvm = dvm.internals().move_vm();
    let _session = mvm.new_session(&adapter, s_id, false);
    Ok(())
}
