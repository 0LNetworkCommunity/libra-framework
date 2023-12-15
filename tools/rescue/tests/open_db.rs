use std::{path::Path, sync::Arc, collections::HashMap, ops::Deref};

use anyhow::bail;

use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_debugger::DiemDebugger;
use diem_storage_interface::{
    cached_state_view::CachedDbStateView,
    state_view::{DbStateView, DbStateViewAtVersion},
    DbReaderWriter, DbReader, MAX_REQUEST_LIMIT,
};
use diem_types::{
    // access_path::AccessPath, account_config::ChainIdResource, account_view::AccountView,
    // move_resource::MoveStorage,
     account_address::AccountAddress, transaction::Version, account_state::AccountState, state_store::{state_key_prefix::StateKeyPrefix, state_key::StateKey, state_value::StateValue},
};
use diem_vm::move_vm_ext::SessionId;
use libra_framework::head_release_bundle;
// use diem_vm::move_vm_ext::SessionId;
use move_core_types::{
    identifier::IdentStr,
    language_storage::{ModuleId, CORE_CODE_ADDRESS, TypeTag, StructTag}, value::{serialize_values, MoveValue}, ident_str,
    // move_resource::MoveStructType,
};
use move_vm_test_utils::gas_schedule::GasStatus;
// use move_vm_test_utils::gas_schedule::GasStatus;

#[tokio::test]
async fn test_debugger() -> anyhow::Result<()> {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");

    let db = DiemDebugger::db(dir)?;

    let v = db.get_latest_version().await?;
    // dbg!(&v);
    // let c = db.get_version_by_account_sequence(CORE_CODE_ADDRESS, 0).await?;
    // dbg!(&c);
    // db.

    let state = db
        .annotate_account_state_at_version(CORE_CODE_ADDRESS, v)
        .await?;
    dbg!(&state);
    Ok(())
}

#[test]
fn test_open() -> anyhow::Result<()>{
    let module_id = ModuleId::new(CORE_CODE_ADDRESS, "account".parse().unwrap());
    let function_name: &IdentStr = IdentStr::new("get_authentication_key").unwrap();

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

    // let state  = get_account_state_by_version(&db_rw.reader, CORE_CODE_ADDRESS, v).unwrap();
    // state.iter().for_each(|(k,v)| {
    //   // dbg!(&k);
    //   match k.inner() {
    //     diem_types::state_store::state_key::StateKeyInner::AccessPath(a) => {
    //       dbg!(&a.get_struct_tag());
    //     },
    //     diem_types::state_store::state_key::StateKeyInner::TableItem { handle, key } => todo!(),
    //     diem_types::state_store::state_key::StateKeyInner::Raw(_) => todo!(),
    // }

    //   // bcs::from_bytes(k.)
    // });
    // state. for_each(|e| {
    // })
    // dbg!(&state);

    let view = db_rw.reader.state_view_at_version(Some(v)).unwrap();

    // // let path = AccessPath::resource_access_path(CORE_CODE_ADDRESS, ChainIdResource::struct_tag()).unwrap();

    // let r = db_rw.reader.as_ref().fetch_resource_by_version(
    //   path,
    //   v
    // ).unwrap();
    // dbg!(&r);

    // let _frozen_state = db_rw
    //     .reader
    //     .get_latest_executed_trees()
    //     .expect("could not get executed state");

    // // let view: CachedDbStateView = DbStateView {
    // //     db: db_rw.reader,
    // //     version: None,
    // // }.into();
    let dvm = diem_vm::DiemVM::new(&view);
    let adapter = dvm.as_move_resolver(&view);

    let s_id = SessionId::Txn {
        sender: CORE_CODE_ADDRESS,
        sequence_number: 0,
        script_hash: b"none".to_vec(),
    };

    let mvm = dvm.internals().move_vm();
    // mvm.new_session(remote, session_id, aggregator_enabled)

    let r = mvm.load_module(&module_id, &adapter).is_ok();

    // dbg!(&r);
    let mut gas_context = GasStatus::new_unmetered();
    let mut session = mvm.new_session(&adapter, s_id, false);
    mvm.mark_loader_cache_as_invalid();
    mvm.flush_loader_cache_if_invalidated();


    let r = mvm.load_module(&module_id, &adapter).is_ok();
    // dbg!(&r);;
    let a = vec![
      MoveValue::Signer(CORE_CODE_ADDRESS)
    ];
    let args = serialize_values(&a);
    // let num = session.execute_entry_function(
    //     &module_id,
    //     &function_name,
    //     vec![],
    //     args,
    //     &mut GasStatus::new_unmetered(),
    // );

    let module_id: ModuleId = ModuleId::new(CORE_CODE_ADDRESS, "tower_state".parse().unwrap());
    let function_name: &IdentStr = IdentStr::new("epoch_param_reset").unwrap();

    let s = StructTag {
      address: CORE_CODE_ADDRESS,
      module: ident_str!("chain_id").into(),
        name: ident_str!("ChainId").into(),
        type_params: vec![],
    };
    // let t = TypeTag::Struct();
    let this_type = session.load_type(&s.clone().into()).unwrap();
    let layout = session.get_type_layout(&s.into()).unwrap();
    let (resource, _) = session.load_resource(CORE_CODE_ADDRESS, &this_type).unwrap();
    let mut a = resource.move_from().unwrap();
    // *a = Value::
    // dbg!(&);

    let new_modules = head_release_bundle();
    println!("publish");
    session.publish_module_bundle_relax_compatibility(new_modules.legacy_copy_code(), CORE_CODE_ADDRESS, &mut gas_context)?;

    // let tag = session.get_type_tag(&this_type).unwrap();
    // tag
    // tag.

    // let res = session.execute_function_bypass_visibility(&module_id, function_name, vec![], args, &mut GasStatus::new_unmetered()).unwrap();
    // dbg!(&res);

    if let Some(pr) = session.extract_publish_request() {
      dbg!(pr.expected_modules.len());
    }

    let module_id: ModuleId = ModuleId::new(CORE_CODE_ADDRESS, "all_your_base".parse().unwrap());
    dbg!(&session.exists_module(&module_id));

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
    let mut iter = db
        .get_prefixed_state_value_iterator(&key_prefix, None, version)?;
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
