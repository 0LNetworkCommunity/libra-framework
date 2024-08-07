use anyhow::{format_err, Context};
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
use libra_config::validator_registration::ValCredentials;
use libra_framework::head_release_bundle;
use move_core_types::{
    ident_str,
    language_storage::{StructTag, CORE_CODE_ADDRESS},
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use std::path::{Path, PathBuf};

// #[derive(Debug, Clone)]
// pub struct ValCredentials {
//     pub account: AccountAddress,
//     pub consensus_pubkey: Vec<u8>,
//     pub proof_of_possession: Vec<u8>,
//     pub network_addresses: Vec<u8>,
//     pub fullnode_addresses: Vec<u8>,
// }

// Run a VM session with a dirty database
// NOTE: there are several implementations of this elsewhere in Diem
// Some are buggy, some don't have exports or APIs needed (DiemDbBootstrapper). Some have issues with async and db locks (DiemDbDebugger).
// so we had to rewrite it.
pub fn libra_run_session<F>(
    dir: PathBuf,
    f: F,
    debug_vals: Option<Vec<AccountAddress>>,
    debug_epoch_interval_microsecs: Option<u64>, // seconds per epoch, for twin and testnet
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

    let framework_sig: MoveValue = MoveValue::Signer(AccountAddress::ONE);
    // if we want to replace the vals, or otherwise use swarm
    // to drive the db state
    if let Some(vals) = debug_vals {
        let vals_cast = MoveValue::vector_address(vals);
        let args = vec![&framework_sig, &vals_cast];
        libra_execute_session_function(&mut session, "0x1::diem_governance::set_validators", args)?;
    }

    // if we want accelerated epochs for twin, testnet, etc
    if let Some(ms) = debug_epoch_interval_microsecs {
        println!("setting epoch interval seconds");
        println!("{}, ms", ms);
        let secs_arg = MoveValue::U64(ms);
        libra_execute_session_function(
            &mut session,
            "0x1::block::update_epoch_interval_microsecs",
            vec![&framework_sig, &secs_arg],
        )
        .expect("set epoch interval seconds");
        libra_execute_session_function(&mut session, "0x1::reconfiguration::reconfigure", vec![])?;
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
) -> anyhow::Result<SerializedReturnValues> {
    let function_tag: StructTag = function_str.parse()?;

    let res = session.execute_function_bypass_visibility(
        &function_tag.module_id(),
        function_tag.name.as_ident_str(),
        function_tag.type_params,
        serialize_values(args),
        &mut UnmeteredGasMeter,
    )?;
    Ok(res)
}

/// Add validators to the session
///
///
/// ! CAUTION: This function will overwrite the current validator set
/// Vector of `ValCredentials` is a list of validators to be added to the session
pub fn session_add_validators(
    session: &mut SessionExt,
    creds: Vec<ValCredentials>,
    upgrade: bool,
) -> anyhow::Result<()> {
    // upgrade the framework
    if upgrade {
        dbg!("upgrade_framework");
        upgrade_framework(session)?;
    }

    // set the chain id (its is set to devnet by default)
    dbg!("set_chain_id");
    libra_execute_session_function(
        session,
        "0x1::chain_id::set_impl",
        vec![&MoveValue::Signer(AccountAddress::ONE), &MoveValue::U8(4)],
    )?;
    // clean the validator universe
    dbg!("clean_validator_universe");
    libra_execute_session_function(
        session,
        "0x1::validator_universe::clean_validator_universe",
        vec![&MoveValue::Signer(AccountAddress::ONE)],
    )?;
    // reset the validators
    dbg!("bulk_set_next_validators");
    libra_execute_session_function(
        session,
        "0x1::stake::bulk_set_next_validators",
        vec![
            &MoveValue::Signer(AccountAddress::ONE),
            &MoveValue::vector_address(vec![]),
        ],
    )?;
    //setup the allowed validators
    for cred in creds.iter() {
        let signer = MoveValue::Signer(AccountAddress::ONE);
        let vector_val = MoveValue::vector_address(vec![cred.account]);
        let args = vec![&signer, &vector_val];
        //configure allowed validators(it should be deprecated??)
        dbg!("configure_allowed_validators");
        libra_execute_session_function(session, "0x1::stake::configure_allowed_validators", args)?;
        let signer = MoveValue::Signer(cred.account);
        let consensus_pubkey = MoveValue::vector_u8(cred.consensus_pubkey.clone());
        let proof_of_possession = MoveValue::vector_u8(cred.proof_of_possession.clone());
        let network_addresses = MoveValue::vector_u8(cred.network_addresses.clone());
        let fullnode_addresses = MoveValue::vector_u8(cred.fullnode_addresses.clone());
        let args = vec![
            &signer,
            &consensus_pubkey,
            &proof_of_possession,
            &network_addresses,
            &fullnode_addresses,
        ];
        //set the initial amount of coins(uts set to 1000)
        let amount = 1000 * 1_000_000_u64;
        let amount = MoveValue::U64(amount);
        //create account
        dbg!("create account");
        libra_execute_session_function(
            session,
            "0x1::ol_account::create_impl",
            vec![&MoveValue::Signer(AccountAddress::ONE), &signer],
        )?;
        //The accounts are not slow so we do not have to unlock them
        dbg!("mint to account");
        libra_execute_session_function(
            session,
            "0x1::libra_coin::mint_to_impl",
            vec![&MoveValue::Signer(AccountAddress::ONE), &signer, &amount],
        )?;
        dbg!("registering validator");
        libra_execute_session_function(
            session,
            "0x1::validator_universe::register_validator",
            args,
        )?;
    }
    let validators = MoveValue::vector_address(creds.iter().map(|c| c.account).collect());
    let signer = MoveValue::Signer(AccountAddress::ONE);
    //set the new validators
    dbg!("set_validators");
    libra_execute_session_function(
        session,
        "0x1::diem_governance::set_validators",
        vec![&signer, &validators],
    )?;
    // RECONFIGURE
    dbg!("on new epoch");
    libra_execute_session_function(session, "0x1::stake::on_new_epoch", vec![])?;
    let vm_signer = MoveValue::Signer(AccountAddress::ZERO);
    dbg!("emit_writeset_block_event");
    libra_execute_session_function(
        session,
        "0x1::block::emit_writeset_block_event",
        vec![&vm_signer, &MoveValue::Address(CORE_CODE_ADDRESS)],
    )?;
    dbg!("reconfigure");
    libra_execute_session_function(session, "0x1::reconfiguration::reconfigure", vec![])?;
    Ok(())
}

/// Unpacks a VM change set.
pub fn unpack_changeset(vmc: VMChangeSet) -> anyhow::Result<ChangeSet> {
    let (write_set, _delta_change_set, events) = vmc.unpack();
    dbg!(&events);
    Ok(ChangeSet::new(write_set, events))
}

/// Publishes the current framework to the database.
pub fn publish_current_framework(
    dir: &Path,
    debug_vals: Option<Vec<AccountAddress>>,
) -> anyhow::Result<ChangeSet> {
    let vmc = libra_run_session(dir.to_path_buf(), combined_steps, debug_vals, None)?;
    unpack_changeset(vmc)
}

fn combined_steps(session: &mut SessionExt) -> anyhow::Result<()> {
    upgrade_framework(session)?;
    writeset_voodoo_events(session)?;
    Ok(())
}

/// Twin testnet registration, replace validator set with new registrations
pub fn twin_testnet(
    dir: &Path,
    testnet_vals: Vec<ValCredentials>,
) -> anyhow::Result<ChangeSet> {
    let vmc = libra_run_session(dir.to_path_buf(), |session| {
      // if we are doing a twin testnet we don't want to upgrade the chain
      session_add_validators(session, testnet_vals, false)
    }, None, None)?;
    unpack_changeset(vmc)
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
