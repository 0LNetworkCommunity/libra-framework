use std::path::Path;

// Database related imports
use diem_db::DiemDB;
use storage_interface::{DbReaderWriter, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD, BUFFERED_STATE_TARGET_ITEMS};
use diem_config::config::{NO_OP_STORAGE_PRUNER_CONFIG, RocksdbConfigs};

// VM related imports
use move_core_types::identifier::ident_str;
use move_core_types::language_storage::{StructTag, CORE_CODE_ADDRESS};

// Session related imports
use diem_types::transaction::SessionId;
use diem_crypto::HashValue;

// Project-specific imports
use crate::session_tools::{
    SessionExt,
    libra_execute_session_function,
    libra_run_session,
    writeset_voodoo_events,
};
use crate::release::{ReleaseTarget, upgrade_framework_head_build};

#[ignore]
#[test]
// test we can publish a db to a fixture
fn test_publish() {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");
    let upgrade_mrb = ReleaseTarget::Head.find_bundle_path().expect("cannot find head.mrb");
    upgrade_framework_head_build(dir, None, &upgrade_mrb).unwrap();
}

// TODO: ability to mutate some state without calling a function
fn _update_resource_in_session(session: &mut SessionExt) {
    let s = StructTag {
        address: CORE_CODE_ADDRESS,
        module: ident_str!("chain_id").into(),
        name: ident_str!("ChainId").into(),
        type_params: vec![],
    };
    let this_type = session.load_type(&s.clone().into()).unwrap();
    let _layout = session.get_type_layout(&s.into()).unwrap();
    let (resource, _) = session
        .load_resource(CORE_CODE_ADDRESS, &this_type)
        .unwrap();
    let _a = resource.move_from().unwrap();
}

#[ignore]
#[test]
// the writeset voodoo needs to be perfect
fn test_voodoo() {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");
    libra_run_session(dir.to_path_buf(), writeset_voodoo_events, None, None).unwrap();
}

#[ignore]
#[test]
// helper to see if an upgraded function is found in the DB
fn test_base() {
    fn check_base(session: &mut SessionExt) -> anyhow::Result<()> {
        libra_execute_session_function(session, "0x1::all_your_base::are_belong_to", vec![])?;
        Ok(())
    }

    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");

    libra_run_session(dir.to_path_buf(), check_base, None, None).unwrap();
}

#[ignore]
#[test]
// testing we can open a database from fixtures, and produce a VM session
fn meta_test_open_db_sync() -> anyhow::Result<()> {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");
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
