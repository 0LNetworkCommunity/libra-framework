use anyhow::{format_err, Context};
use std::path::Path;

use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_gas::{ChangeSetConfigs, LATEST_GAS_FEATURE_VERSION};
use diem_storage_interface::{state_view::DbStateViewAtVersion, DbReaderWriter};
use diem_types::{account_address::AccountAddress, transaction::ChangeSet};
use diem_vm::move_vm_ext::{MoveVmExt, SessionExt, SessionId};
use diem_vm_types::change_set::VMChangeSet;
use libra_framework::head_release_bundle;
use move_core_types::{
    ident_str,
    language_storage::{StructTag, CORE_CODE_ADDRESS},
    value::{serialize_values, MoveValue},
};

use move_vm_types::gas::UnmeteredGasMeter;

// Run a VM session with a dirty database
// NOTE: there are several implementations of this elsewhere in Diem
// Some are buggy, some don't have exports or APIs needed (DiemDbBootstrapper). Some have issues with async and db locks (DiemDbDebugger).
// so we had to rewrite it.
pub fn libra_run_session<F>(
    dir: &Path,
    f: F,
    debug_vals: Option<Vec<AccountAddress>>,
) -> anyhow::Result<VMChangeSet>
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

    let view = db_rw.reader.state_view_at_version(Some(v)).unwrap();
    let dvm = diem_vm::DiemVM::new(&view);
    let adapter = dvm.as_move_resolver(&view);
    let s_id = SessionId::genesis(diem_crypto::HashValue::zero());
    let mvm: &MoveVmExt = dvm.internals().move_vm();

    let mut session = mvm.new_session(&adapter, s_id, false);

    ////// FUNCTIONS RUN HERE
    f(&mut session)
        .map_err(|err| format_err!("Unexpected VM Error Running Rescue VM Session: {:?}", err))?;
    //////

    // if we want to replace the vals, or otherwise use swarm
    // to drive the db state
    if let Some(vals) = debug_vals {
        let vm_signer = MoveValue::Signer(AccountAddress::ZERO);
        let vals_cast = MoveValue::vector_address(vals);
        let args = vec![&vm_signer, &vals_cast];
        libra_execute_session_function(&mut session, "0x1::stake::maybe_reconfigure", args)?;
    }

    let change_set = session.finish(
        &mut (),
        &ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION),
    )?;

    println!("session run sucessfully");
    Ok(change_set)
}

// BLACK MAGIC
// there's a bunch of branch magic that happens for a writeset.
// these are the ceremonial dance steps
// don't upset the gods
pub fn writeset_voodoo_events(session: &mut SessionExt) -> anyhow::Result<()> {
    libra_execute_session_function(session, "0x1::stake::on_new_epoch", vec![])?;

    let vm_signer = MoveValue::Signer(AccountAddress::ZERO);

    libra_execute_session_function(
        session,
        "0x1::block::emit_writeset_block_event",
        vec![
            &vm_signer,
            // note: any address would work below
            &MoveValue::Address(CORE_CODE_ADDRESS),
        ],
    )?;

    ////// TODO: revert this once rescue is complete
    // libra_execute_session_function(
    //     session,
    //     "0x1::reconfiguration::reconfigure_for_rescue",
    //     vec![&vm_signer],
    // )?;
    //////

    libra_execute_session_function(session, "0x1::reconfiguration::reconfigure", vec![])?;

    Ok(())
}

// wrapper to publish the latest head framework release
pub fn upgrade_framework(session: &mut SessionExt) -> anyhow::Result<()> {
    let new_modules = head_release_bundle();

    session.publish_module_bundle_relax_compatibility(
        new_modules.legacy_copy_code(),
        CORE_CODE_ADDRESS,
        &mut UnmeteredGasMeter,
    )?;
    Ok(())
}

// wrapper to exectute a function
// call anythign you want, except #[test] functions
// note this ignores the `public` and `friend` visibility
pub fn libra_execute_session_function(
    session: &mut SessionExt,
    function_str: &str,
    args: Vec<&MoveValue>,
) -> anyhow::Result<()> {
    let function_tag: StructTag = function_str.parse()?;

    // TODO: return Serialized Values
    let _res = session.execute_function_bypass_visibility(
        &function_tag.module_id(),
        function_tag.name.as_ident_str(),
        function_tag.type_params,
        serialize_values(args),
        &mut UnmeteredGasMeter,
    )?;
    Ok(())
}

///  drops a users from the chain
pub fn drop_users(dir: &Path, addr_list: &[AccountAddress]) -> anyhow::Result<ChangeSet> {
    let vm_sig = MoveValue::Signer(AccountAddress::ZERO);
    let vmc = libra_run_session(
        dir,
        move |session| {
            upgrade_framework(session).expect("upgrade framework");

            addr_list.iter().for_each(|a| {
                let user: MoveValue = MoveValue::Signer(*a);
                libra_execute_session_function(
                    session,
                    "0x1::last_goodbye::dont_think_twice_its_alright",
                    vec![&vm_sig, &user],
                )
                .expect("run through whole list");
            });

            writeset_voodoo_events(session).expect("voodoo");
            Ok(())
        },
        None,
    )
    .expect("could run session");

    unpack_changeset(vmc)
}

pub fn unpack_changeset(vmc: VMChangeSet) -> anyhow::Result<ChangeSet> {
    let (write_set, _delta_change_set, events) = vmc.unpack();
    Ok(ChangeSet::new(write_set, events))
}

pub fn publish_current_framework(
    dir: &Path,
    debug_vals: Option<Vec<AccountAddress>>,
) -> anyhow::Result<ChangeSet> {
    let vmc = libra_run_session(dir, combined_steps, debug_vals)?;
    unpack_changeset(vmc)
}

fn combined_steps(session: &mut SessionExt) -> anyhow::Result<()> {
    upgrade_framework(session)?;
    writeset_voodoo_events(session)?;
    Ok(())
}

#[ignore]
#[test]
// test we can publish a db to a fixture
fn test_publish() {
    let dir = Path::new("/root/dbarchive/data_bak_2023-12-11/db");

    publish_current_framework(dir, None).unwrap();
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
    libra_run_session(dir, writeset_voodoo_events, None).unwrap();
}

#[test]
// helper to see if an upgraded function is found in the DB
fn test_drop_user() {
    let addr: AccountAddress = "0x7B61439A88060096213AC4F5853B598E".parse().unwrap();
    let dir = Path::new("/root/db");
    drop_users(dir, &[addr]).expect("drop user");
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
