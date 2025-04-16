// This module tests that a pure restored db
// (using only the backup files from `epoch-archive-mainnet`)
// can be used for offline writesets (rescue missions), including
// twin testnets.

// WIP
// There is a known issue with the restored DBs at exact epoch boundaries
// which is not working. There is some voodoo which is not
// being invoked, the epoch event is not being emitted.

use std::env;
use std::path::PathBuf;

use anyhow::Context;
use diem_storage_interface::state_view::DbStateViewAtVersion;
use diem_storage_interface::DbReaderWriter;
use diem_vm::move_vm_ext::SessionId;
use libra_framework::release::ReleaseTarget;
use libra_rescue::cli_bootstrapper::BootstrapOpts;
use libra_rescue::cli_main::RescueCli;
use libra_rescue::cli_main::Sub;
use libra_rescue::test_support::setup_v7_reference_twin_db;

// Database related imports
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;

use libra_rescue::cli_main::REPLACE_VALIDATORS_BLOB;
use libra_rescue::cli_main::UPGRADE_FRAMEWORK_BLOB;
// VM related imports
use move_core_types::language_storage::CORE_CODE_ADDRESS;

// Project-specific imports
use libra_rescue::session_tools::{
    libra_run_session, upgrade_framework_changeset, writeset_voodoo_events,
};

#[test]
/// The writeset voodoo needs to be perfect
///
/// Uses a database fixture extracted from `./rescue/fixtures/db_339.tar.gz`
fn test_voodoo_on_v7() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;
    libra_run_session(dir, writeset_voodoo_events, None, None)?;
    Ok(())
}

#[test]
/// Testing we can open a database from fixtures, and produce a VM session
///
/// Uses a database fixture extracted from `./rescue/fixtures/db_339.tar.gz`
fn meta_test_open_db_on_restored_v7() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;

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

#[test]
/// Test we can publish a framework to a database fixture
///
/// Uses a database fixture extracted from `./rescue/fixtures/db_339.tar.gz`
fn publish_on_restored_v7_changeset_success() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;
    let upgrade_mrb = ReleaseTarget::Head
        .find_bundle_path()
        .expect("cannot find head.mrb");

    upgrade_framework_changeset(&dir, None, &upgrade_mrb)?;

    Ok(())
}

// take a recovered db using only the backup files from `epoch-archive-mainnet`
// and try to upgrade and bootstrap a db
/// using the command-line entry point
#[test]
fn e2e_publish_on_restored_v7() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;
    let blob_path = dir.clone();
    let bundle = ReleaseTarget::Head
        .find_bundle_path()
        .expect("cannot find head.mrb");

    let r = RescueCli {
        db_path: dir.clone(),
        blob_path: Some(blob_path.clone()),
        command: Sub::UpgradeFramework {
            upgrade_mrb: bundle,
            set_validators: None,
        },
    };

    r.run().context("rescue tx fails")?;

    let file = blob_path.join(UPGRADE_FRAMEWORK_BLOB);
    assert!(file.exists());

    println!(
        "3. check we can apply the tx to existing db, and can get a waypoint, don't commit it"
    );

    let boot = BootstrapOpts {
        db_dir: dir,
        genesis_txn_file: file,
        waypoint_to_verify: None,
        commit: false,
        update_node_config: None,
        info: false,
    };

    let wp = boot.run()?;
    assert!(
        wp.unwrap().version()
          // do not use hash to test since it will change on evert Move build
            == 117583051u64,
        "wrong waypoint"
    );

    Ok(())
}

/// Same as above, but also register some validators
/// and upgrade the framework
#[test]
fn e2e_twin_register_vals_plus_upgrade_on_v7() -> anyhow::Result<()> {
    let dir = setup_v7_reference_twin_db()?;
    let blob_path = dir.clone();
    // get the Alice operator.yaml from fixtures
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let oper_file = PathBuf::from(manifest_dir).join("fixtures/operator.yaml");

    let upgrade_mrb = ReleaseTarget::Head
        .find_bundle_path()
        .expect("cannot find head.mrb");

    let r = RescueCli {
        db_path: dir.clone(),
        blob_path: Some(blob_path.clone()),
        command: Sub::RegisterVals {
            operator_yaml: vec![oper_file],
            upgrade_mrb: Some(upgrade_mrb),
            chain_id: None,
        },
    };

    r.run()?;

    let file = blob_path.join(REPLACE_VALIDATORS_BLOB);
    assert!(file.exists());

    println!(
        "3. check we can apply the tx to existing db, and can get a waypoint, don't commit it"
    );

    let boot = BootstrapOpts {
        db_dir: dir,
        genesis_txn_file: file,
        waypoint_to_verify: None,
        commit: false,
        update_node_config: None,
        info: false,
    };

    let wp = boot.run()?;
    assert!(
        wp.unwrap().version()
          // do not use hash to test since it will change on evert Move build
            == 117583051u64,
        "wrong waypoint"
    );

    // hack, getting weird sigabrt on exit
    std::thread::sleep(std::time::Duration::from_millis(10));
    Ok(())
}
