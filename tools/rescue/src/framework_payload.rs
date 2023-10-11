use diem_debugger::DiemDebugger;
use diem_types::{
    account_config::CORE_CODE_ADDRESS,
    transaction::{ChangeSet, WriteSetPayload},
};
use move_core_types::value::MoveValue;
use move_vm_test_utils::gas_schedule::GasStatus;
use std::path::PathBuf;

/// generate the writeset of changes from publishing all a framework bundle
pub async fn stlib_payload(db_path: PathBuf) -> anyhow::Result<WriteSetPayload> {
    let db = DiemDebugger::db(db_path)?;

    // publish the agreed stdlib
    let new_stdlib = libra_framework::head_release_bundle().legacy_copy_code();

    let v = db.get_latest_version().await?;
    let cs = db.run_session_at_version(v, |session| {
        let mut gas_status = GasStatus::new_unmetered();
        session.publish_module_bundle(new_stdlib, CORE_CODE_ADDRESS, &mut gas_status)
    })?;

    let (ws, _, events) = cs.unpack();
    let other_changset_type_fml = ChangeSet::new(ws, events);
    Ok(WriteSetPayload::Direct(other_changset_type_fml))
}

/// generate the writeset of changes from publishing all a framework bundle
pub async fn execute_script_payload(
    db_path: PathBuf,
    script: Vec<u8>,
) -> anyhow::Result<WriteSetPayload> {
    let db = DiemDebugger::db(db_path)?;

    // publish the agreed stdlib
    // let new_stdlib = libra_framework::head_release_bundle().legacy_copy_code();

    let v = db.get_latest_version().await?;
    let cs = db.run_session_at_version(v, |session| {
        let mut gas_status = GasStatus::new_unmetered();

        // session
        // .publish_module_bundle(new_stdlib, CORE_CODE_ADDRESS, &mut
        // gas_status)
        // just the signer
        let args = vec![MoveValue::Signer(CORE_CODE_ADDRESS)
            .simple_serialize()
            .unwrap()];

        session
            .execute_script(script, vec![], args, &mut gas_status)
            .expect("could not execute script");
        Ok(())
    })?;
    let (ws, _, events) = cs.unpack();
    let other_changset_type_fml = ChangeSet::new(ws, events);
    Ok(WriteSetPayload::Direct(other_changset_type_fml))
}
