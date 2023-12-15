use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{bail, format_err, Context};

use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_gas::{ChangeSetConfigs, LATEST_GAS_FEATURE_VERSION};
use diem_storage_interface::{
    state_view::DbStateViewAtVersion, DbReader, DbReaderWriter, MAX_REQUEST_LIMIT,
};
use diem_types::{
    account_address::AccountAddress,
    state_store::{state_key::StateKey, state_key_prefix::StateKeyPrefix, state_value::StateValue},
    transaction::{ChangeSet, Version},
};
use diem_vm::move_vm_ext::{MoveVmExt, SessionExt, SessionId};
use diem_vm_types::change_set::VMChangeSet;
use libra_framework::head_release_bundle;
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, StructTag, CORE_CODE_ADDRESS},
    value::{serialize_values, MoveValue},
};

use move_vm_types::gas::UnmeteredGasMeter;

pub fn publish_current_framework(dir: &Path) -> anyhow::Result<ChangeSet> {
    // let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");
    let db = DiemDB::open(
        dir,
        true,
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner */
        RocksdbConfigs::default(),
        false, /* indexer */
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    )
    .context("Failed to open DB.")?;

    let db_rw = DbReaderWriter::new(db);

    let v = db_rw.reader.get_latest_version().unwrap();
    dbg!(&v);

    let view = db_rw.reader.state_view_at_version(Some(v)).unwrap();

    let dvm = diem_vm::DiemVM::new(&view);
    let adapter = dvm.as_move_resolver(&view);

    let s_id = SessionId::genesis(diem_crypto::HashValue::zero());

    let mvm = dvm.internals().move_vm();

    let mut gas_context = UnmeteredGasMeter;
    let mut session = mvm.new_session(&adapter, s_id, false);

    let new_module_id: ModuleId =
        ModuleId::new(CORE_CODE_ADDRESS, "all_your_base".parse().unwrap());

    let res = session.execute_function_bypass_visibility(
        &new_module_id,
        ident_str!("are_belong_to"),
        vec![],
        serialize_values(vec![]),
        &mut gas_context,
    );
    assert!(res.is_err());

    let new_modules = head_release_bundle();
    println!("publish");
    session.publish_module_bundle_relax_compatibility(
        new_modules.legacy_copy_code(),
        CORE_CODE_ADDRESS,
        &mut gas_context,
    )?;

    let new_module_id: ModuleId =
        ModuleId::new(CORE_CODE_ADDRESS, "all_your_base".parse().unwrap());

    let res = session.execute_function_bypass_visibility(
        &new_module_id,
        ident_str!("are_belong_to"),
        vec![],
        serialize_values(vec![]),
        &mut gas_context,
    );
    dbg!(&res);

    // let (a, b, ..) = session.finish()?;
    let change_set = session
        .finish(
            &mut (),
            &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
        )
        .context("Failed to generate txn effects")?;
    // TODO: Support deltas in fake executor.
    let (write_set, _delta_change_set, events) = change_set.unpack();
    let change_set = ChangeSet::new(write_set, events);
    Ok(change_set)
}

#[test]
fn test_voodoo() {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");
    libra_run_session(dir, writeset_voodoo_events).unwrap();
}

pub fn libra_run_session<F>(dir: &Path, f: F) -> anyhow::Result<VMChangeSet>
where
    F: FnOnce(&mut SessionExt) -> anyhow::Result<()>,
{
    let db = DiemDB::open(
        dir,
        true,
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner */
        RocksdbConfigs::default(),
        false, /* indexer */
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    )
    .context("Failed to open DB.")
    .unwrap();

    let db_rw = DbReaderWriter::new(db);

    let v = db_rw.reader.get_latest_version().unwrap();
    dbg!(&v);

    let view = db_rw.reader.state_view_at_version(Some(v)).unwrap();
    let dvm = diem_vm::DiemVM::new(&view);
    let adapter = dvm.as_move_resolver(&view);
    let s_id = SessionId::genesis(diem_crypto::HashValue::zero());
    let mvm: &MoveVmExt = dvm.internals().move_vm();

    // let mut gas_context = UnmeteredGasMeter;
    let mut session = mvm.new_session(&adapter, s_id, false);

    ////// FUNCTIONS RUN HERE
    f(&mut session)
        .map_err(|err| format_err!("Unexpected VM Error Running Rescue VM Session: {:?}", err))?;
    //////

    let change_set = session.finish(
        &mut (),
        &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
    )?;
    Ok(change_set)
}

pub fn writeset_voodoo_events(session: &mut SessionExt) -> anyhow::Result<()> {
    libra_execute_session_function(session, "0x1::reconfiguration::reconfigure", vec![])
}

pub fn upgrade_framework(session: &mut SessionExt) -> anyhow::Result<()> {
    let new_modules = head_release_bundle();

    session.publish_module_bundle_relax_compatibility(
        new_modules.legacy_copy_code(),
        CORE_CODE_ADDRESS,
        &mut UnmeteredGasMeter,
    )?;
    Ok(())
}
pub fn libra_execute_session_function(
    session: &mut SessionExt,
    function_str: &str,
    args: Vec<&MoveValue>,
) -> anyhow::Result<()> {
    let function_tag: StructTag = function_str.parse()?;

    let _res = session.execute_function_bypass_visibility(
        &function_tag.module_id(),
        function_tag.name.as_ident_str(),
        function_tag.type_params,
        serialize_values(args),
        &mut UnmeteredGasMeter,
    );

    Ok(())
}

// pub fn libra_finish_session(session: &mut SessionExt) -> anyhow::Result<ChangeSet> {
//     let change_set = session
//         .finish(
//             &mut (),
//             &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
//         )
//         .context("Failed to generate txn effects")?;
//     let (write_set, _delta_change_set, events) = change_set.unpack();

//     Ok(ChangeSet::new(write_set, events))
// }

#[test]
fn test_publish() {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");
    let _ws = publish_current_framework(dir).unwrap();
}

// fn update_resource_in_session(session: &mut SessionExt) {
//     let s = StructTag {
//         address: CORE_CODE_ADDRESS,
//         module: ident_str!("chain_id").into(),
//         name: ident_str!("ChainId").into(),
//         type_params: vec![],
//     };
//     let this_type = session.load_type(&s.clone().into()).unwrap();
//     let layout = session.get_type_layout(&s.into()).unwrap();
//     let (resource, _) = session
//         .load_resource(CORE_CODE_ADDRESS, &this_type)
//         .unwrap();
//     let mut a = resource.move_from().unwrap();
// }
// #[tokio::test]
// async fn test_debugger() -> anyhow::Result<()> {
//     let dir: &Path = Path::new("/root/dbarchive/data_bak_2023-12-11/db");

//     let db = DiemDebugger::db(dir)?;

//     let v = db.get_latest_version().await?;
//     // dbg!(&v);
//     // let c = db.get_version_by_account_sequence(CORE_CODE_ADDRESS, 0).await?;
//     // dbg!(&c);
//     // db.

//     let state = db
//         .annotate_account_state_at_version(CORE_CODE_ADDRESS, v)
//         .await?;
//     dbg!(&state);
//     Ok(())
// }

#[test]
fn test_open() -> anyhow::Result<()> {
    use move_core_types::identifier::IdentStr;
    use move_vm_test_utils::gas_schedule::GasStatus;
    // let module_id = ModuleId::new(CORE_CODE_ADDRESS, "account".parse().unwrap());
    // let function_name: &IdentStr = IdentStr::new("get_authentication_key").unwrap();

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
    dbg!(&v);

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
    let mut gas_context = GasStatus::new_unmetered();
    let mut session = mvm.new_session(&adapter, s_id, false);

    let a = vec![MoveValue::Signer(CORE_CODE_ADDRESS)];
    let _args = serialize_values(&a);

    let _module_id: ModuleId = ModuleId::new(CORE_CODE_ADDRESS, "tower_state".parse().unwrap());
    let _function_name: &IdentStr = IdentStr::new("epoch_param_reset").unwrap();

    let new_modules = head_release_bundle();
    println!("publish");
    session.publish_module_bundle_relax_compatibility(
        new_modules.legacy_copy_code(),
        CORE_CODE_ADDRESS,
        &mut gas_context,
    )?;
    // session.

    // let tag = session.get_type_tag(&this_type).unwrap();
    // tag
    // tag.

    // if let Some(pr) = session.extract_publish_request() {
    //     dbg!(pr.expected_modules.len());
    // }

    let new_module_id: ModuleId =
        ModuleId::new(CORE_CODE_ADDRESS, "all_your_base".parse().unwrap());

    let res = session.execute_function_bypass_visibility(
        &new_module_id,
        ident_str!("are_belong_to"),
        vec![],
        serialize_values(vec![]),
        &mut gas_context,
    );
    dbg!(&res);
    // dbg!(&session.exists_module(&new_module_id));

    // session.exists_module(&);

    // let e = session.exists_module(&module_id).unwrap();
    // dbg!(&p_r.destination);
    // let converter = adapter.as_converter(db_rw.reader);

    // let move_vm = d.internals().move_vm();

    // move_vm.new_session(&view, session_id, aggregator_enabled)
    // dbg!(&cache_invalid);
    Ok(())
}

fn get_account_state_by_version(
    db: &Arc<dyn DbReader>,
    account: AccountAddress,
    version: Version,
) -> anyhow::Result<HashMap<StateKey, StateValue>> {
    let key_prefix = StateKeyPrefix::from(account);
    let mut iter = db.get_prefixed_state_value_iterator(&key_prefix, None, version)?;
    // dbg!(&iter.by_ref().count());
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
    Ok(kvs)
}
