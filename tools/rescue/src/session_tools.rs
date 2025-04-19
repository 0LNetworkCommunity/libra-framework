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

    // voodoo time travel
    // say a prayer
    // before the writeset
    // or the gods will be angry
    // and the writeset will fail later
    // once you try to bootstrap it
    // with INVALID WRITESET
    voodoo_time_travel(&mut session)?;

    ////// FUNCTIONS RUN HERE
    f(&mut session)
        .map_err(|err| format_err!("Unexpected VM Error Running Rescue VM Session: {:?}", err))?;
    //////

    let framework_sig: MoveValue = MoveValue::Signer(AccountAddress::ONE);
    // if we want to replace the vals the vals must ALREADY be registered
    // in the db we are running a session on, and bootstrapping
    if let Some(vals) = debug_vals {
        let vals_cast = MoveValue::vector_address(vals);
        let args = vec![&framework_sig, &vals_cast];
        libra_execute_session_function(&mut session, "0x1::diem_governance::set_validators", args)?;
    }

    // if we want accelerated epochs for twin, testnet, etc
    if let Some(ms) = debug_epoch_interval_microsecs {
        println!("[vm session] setting epoch interval seconds");
        println!("[vm session] {}, ms", ms);
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

    println!("[vm session] session run sucessfully");
    Ok(change_set)
}

// wrapper to execute a function
// call anything you want, except #[test] functions
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
        match libra_execute_session_function(
            session,
            "0x1::ol_account::create_impl",
            vec![&MoveValue::Signer(AccountAddress::ONE), &signer],
        ) {
            Ok(_) => {
                println!("[vm session] account created successfully");
                //The accounts are not slow so we do not have to unlock them
                libra_execute_session_function(
                    session,
                    "0x1::libra_coin::mint_to_impl",
                    vec![&MoveValue::Signer(AccountAddress::ONE), &signer, &amount],
                )?;
                libra_execute_session_function(
                    session,
                    "0x1::validator_universe::register_validator",
                    args,
                )?;
            }
            Err(_) => {
                println!("[vm session] account already exists, skipping");
            }
        };
    }
    let validators = MoveValue::vector_address(creds.iter().map(|c| c.account).collect());
    let signer = MoveValue::Signer(AccountAddress::ONE);
    //set the new validators
    libra_execute_session_function(
        session,
        "0x1::diem_governance::set_validators",
        vec![&signer, &validators],
    )?;

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

/// change the chain-id,
/// shouldn't be using mainnet in tests, possibility
/// of making testers have a bad day, and small chance
/// of creating replayable artifacts if using chain_id = 1
pub fn change_chain_id(session: &mut SessionExt, chain_id: u8) -> anyhow::Result<()> {
    let signer = MoveValue::Signer(AccountAddress::ONE);
    let chain_id = MoveValue::U8(chain_id);
    libra_execute_session_function(session, "0x1::chain_id::set_impl", vec![&signer, &chain_id])?;

    libra_execute_session_function(
        session,
        "0x1::block::update_epoch_interval_microsecs",
        // 5 minutes
        // TODO: get from globals
        vec![&signer, &MoveValue::U64(5 * 60 * 1_000_000)],
    )?;

    // TODO: uncomment this after v8
    // libra_execute_session_function(
    //     session,
    //     "0x1::block::set_interval_with_global",
    //     vec![&signer],
    // )?;
    Ok(())
}

/// for twin testnets we usually want to be out of governance
/// mode in case we are in it
pub fn remove_gov_mode_flag(session: &mut SessionExt) -> anyhow::Result<()> {
    let signer = MoveValue::Signer(AccountAddress::ONE);
    let add_features = MoveValue::Vector(vec![]);
    // note check ol_features.move for the integer, currently: 25
    let remove_features = MoveValue::Vector(vec![MoveValue::U64(25)]);
    libra_execute_session_function(
        session,
        "0x1::features::change_feature_flags",
        vec![&signer, &add_features, &remove_features],
    )?;
    Ok(())
}

// TODO: this may be useful, but it won't run newly created migrations.
// pub fn migrate_data(session: &mut SessionExt) -> anyhow::Result<()> {
//     let signer = MoveValue::Signer(AccountAddress::ONE);
//     libra_execute_session_function(session, "0x1::epoch_boundary::migrate_data", vec![&signer])?;
//     Ok(())
// }

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
    chain_id: Option<u8>, // will default to staging chain_id = 2
) -> anyhow::Result<ChangeSet> {
    let vmc = libra_run_session(
        dir.to_path_buf(),
        |session| {
            if let Some(p) = upgrade_mrb {
                upgrade_framework_from_mrb_file(session, p).expect("could not upgrade framework");
            }

            session_register_validators(session, replacement_vals)
                .expect("could not register validators");

            change_chain_id(session, chain_id.unwrap_or(2)).expect("could not change chain id");

            remove_gov_mode_flag(session).expect("could not remove governance mode");

            writeset_voodoo_events(session).expect("should voodoo, who do?");

            Ok(())
        },
        None, // uses the validators registered above
        None,
    )?;
    unpack_to_changeset(vmc)
}

///////////////// BEGIN BLACK MAGIC /////////////////
// __________.____       _____  _________  ____  __.
// \______   \    |     /  _  \ \_   ___ \|    |/ _|
//  |    |  _/    |    /  /_\  \/    \  \/|      <
//  |    |   \    |___/    |    \     \___|    |  \
//  |______  /_______ \____|__  /\______  /____|__ \
//         \/        \/       \/        \/        \/
//    _____      _____    ________.____________
//   /     \    /  _  \  /  _____/|   \_   ___ \
//  /  \ /  \  /  /_\  \/   \  ___|   /    \  \/
// /    Y    \/    |    \    \_\  \   \     \____
// \____|__  /\____|__  /\______  /___|\______  /
//         \/         \/        \/            \/
//
// Offline writesets are a second class. There's a bunch of branch magic
// that Diem code does to obviate the checks the VM framework relies on
// while in production.
// e.g. reconfigure.move "don't reconfigure if the time hasn't
// changed since last time you did this".
// don't emit new block events willynilly.
// However a rescue blob (or genesis blob) requires these events emitted.
// So the writeset tooling in diem and vendor, does this black magic stuff (yes we asked them, and there's no other way).
// It's particularly a problem for testing. We would like to be able to recover a backup of a specific epoch from archive files.
// While an epoch restore contains all the events needed, applying a writeset
// directly to it will fail.
// Because the time hasn't changed, or there's no way to make a block event.
// Anyhow, short of neuromancy and a lot of trial and error, we have to do this.
// Loup garou, loup garou, loup garou.

pub fn voodoo_time_travel(session: &mut SessionExt) -> anyhow::Result<()> {
    // DEV NOTE: in future versions of the libra framework we
    // may make this easier. Currently this is the only sequence of
    // keystrokes that will change the time on v7 mainnet.
    let res = libra_execute_session_function(session, "0x1::timestamp::now_microseconds", vec![])?;

    let mut last_timestamp_in_db_ms: u64 = 0;
    res.return_values.iter().for_each(|v| {
        let secs = MoveValue::simple_deserialize(&v.0, &v.1)
            .expect("to parse timestamp::now_microseconds");
        println!("[vm session] now_microseconds: {}", secs);
        if let MoveValue::U64(s) = secs {
            if s > last_timestamp_in_db_ms {
                last_timestamp_in_db_ms = s;
                println!("[vm session] last_time_in_db: {}", s);
            }
        }
    });

    let signer = MoveValue::Signer(AccountAddress::ZERO);
    // proposer must be different than VM 0x0, otherwise time will not advance.
    let addr = MoveValue::Address(AccountAddress::ONE);

    // always use prime numbers when voodoo is involved
    // █ ▄▄  █▄▄▄▄ ▄█ █▀▄▀█ ▄███▄
    // █   █ █  ▄▀ ██ █ █ █ █▀   ▀
    // █▀▀▀  █▀▀▌  ██ █ ▄ █ ██▄▄
    // █     █  █  ▐█ █   █ █▄   ▄▀
    //  █      █    ▐    █  ▀███▀
    //   ▀    ▀         ▀

    //    ▄     ▄   █▀▄▀█ ███   ▄███▄   █▄▄▄▄
    //     █     █  █ █ █ █  █  █▀   ▀  █  ▄▀
    // ██   █ █   █ █ ▄ █ █ ▀ ▄ ██▄▄    █▀▀▌
    // █ █  █ █   █ █   █ █  ▄▀ █▄   ▄▀ █  █
    // █  █ █ █▄ ▄█    █  ███   ▀███▀     █
    // █   ██  ▀▀▀    ▀                  ▀

    let new_timestamp = MoveValue::U64(last_timestamp_in_db_ms + 7);

    libra_execute_session_function(
        session,
        "0x1::timestamp::update_global_time",
        vec![&signer, &addr, &new_timestamp],
    )?;
    Ok(())
}

// there's a bunch of branch magic that happens for a writeset.
// These event are for the block event and epoch boundary
//  (via reconfiguration.move).
// TODO: its not clear as of yet if the time travel needs to happen before the
// script, or could happen here.

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
//////// END BLACK MAGIC ////////
