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
use libra_framework::release::ReleaseTarget;
use move_core_types::{
    language_storage::{StructTag, CORE_CODE_ADDRESS},
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use std::path::{Path, PathBuf};

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
    .context("failed to open db")?;
    let db_rw = DbReaderWriter::new(db);
    let v = db_rw.reader.get_latest_version()?;
    let view = db_rw.reader.state_view_at_version(Some(v))?;
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
pub fn session_register_validators(
    session: &mut SessionExt,
    creds: Vec<ValCredentials>,
) -> anyhow::Result<()> {
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
        match libra_execute_session_function(
            session,
            "0x1::ol_account::create_impl",
            vec![&MoveValue::Signer(AccountAddress::ONE), &signer],
        ) {
            Ok(_) => {
                println!("account created successfully");
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
            Err(_) => {
                println!("account already exists, skipping");
            }
        };
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

    dbg!("end session");
    Ok(())
}

// wrapper to publish the latest head framework release
pub fn upgrade_framework_from_mrb_file(
    session: &mut SessionExt,
    upgrade_mrb: &Path,
) -> anyhow::Result<()> {
    let new_modules = ReleaseTarget::load_bundle_from_file(upgrade_mrb.to_path_buf())?;

    session.publish_module_bundle_relax_compatibility(
        new_modules.legacy_copy_code(),
        CORE_CODE_ADDRESS,
        &mut UnmeteredGasMeter,
    )?;
    Ok(())
}

/// Unpacks a VM change set.
pub fn unpack_to_changeset(vmc: VMChangeSet) -> anyhow::Result<ChangeSet> {
    let (write_set, _delta_change_set, events) = vmc.unpack();

    Ok(ChangeSet::new(write_set, events))
}

/// Publishes the current framework to the database.
pub fn upgrade_framework_changeset(
    dir: &Path,
    debug_vals: Option<Vec<AccountAddress>>,
    upgrade_mrb: &Path,
) -> anyhow::Result<ChangeSet> {
    let vmc = libra_run_session(
        dir.to_path_buf(),
        |session| {
            upgrade_framework_from_mrb_file(session, upgrade_mrb)
                .expect("should publish framework");
            writeset_voodoo_events(session).expect("should voodoo, who do?");
            Ok(())
        },
        debug_vals,
        None,
    )?;
    unpack_to_changeset(vmc)
}

/// Twin testnet registration, replace validator set with new registrations
pub fn register_and_replace_validators_changeset(
    dir: &Path,
    replacement_vals: Vec<ValCredentials>,
    upgrade_mrb: &Option<PathBuf>,
) -> anyhow::Result<ChangeSet> {
    let vmc = libra_run_session(
        dir.to_path_buf(),
        |session| {
            if let Some(p) = upgrade_mrb {
                upgrade_framework_from_mrb_file(session, p).expect("could not upgrade framework");
            }

            session_register_validators(session, replacement_vals)
                .expect("could not register validators");

            writeset_voodoo_events(session).expect("should voodoo, who do?");

            Ok(())
        },
        None, // uses the validators registered above
        None,
    )?;
    unpack_to_changeset(vmc)
}
