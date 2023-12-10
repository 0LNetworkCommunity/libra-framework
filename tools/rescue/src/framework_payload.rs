use diem_debugger::DiemDebugger;
use diem_types::{
    account_address::AccountAddress,
    account_config::CORE_CODE_ADDRESS,
    transaction::{ChangeSet, WriteSetPayload},
};
use move_core_types::{identifier::Identifier, language_storage::ModuleId, value::MoveValue};
use move_vm_test_utils::gas_schedule::GasStatus;
use std::path::PathBuf;

use diem_vm::move_vm_ext::SessionExt;
use move_core_types::language_storage::TypeTag;
use move_core_types::value::serialize_values;

/// generate the writeset of changes from publishing all a framework bundle
pub async fn stlib_payload(db_path: PathBuf) -> anyhow::Result<WriteSetPayload> {
    let db = DiemDebugger::db(db_path)?;

    // publish the agreed stdlib
    let new_stdlib = libra_framework::head_release_bundle().legacy_copy_code();

    let v = db.get_latest_version().await?;
    dbg!(&v);

    let cs = db.run_session_at_version(v, |session| {
        let mut gas_status = GasStatus::new_unmetered();
        dbg!("disable reconfig");
        // this is a hack.
        // diem-node doesn't accept reconfigurations where the
        // validator set has not changed.
        // BUT writesets would oridinary require reconfigurations
        // to be valid. Unless we disable the reconfiguration feature
        disable_reconfiguration(session);

        session
            .publish_module_bundle(new_stdlib, CORE_CODE_ADDRESS, &mut gas_status)
            .expect("could not publish framework");
        // turn it back on to resume normal production mode
        enable_reconfiguration(session);
        Ok(())
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

    let v = db.get_latest_version().await?;
    let cs = db.run_session_at_version(v, |session| {
        let mut gas_status = GasStatus::new_unmetered();
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

// NOTE this is adapted from Diem
// diem-move/writeset-transaction-generator/src/writeset_builder.rs

// helper to execute a session function
pub fn exec_function_helper(
    session: &mut SessionExt,
    module_name: &str,
    function_name: &str,
    ty_args: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
) {
    session
        .execute_function_bypass_visibility(
            &ModuleId::new(AccountAddress::ONE, Identifier::new(module_name).unwrap()),
            &Identifier::new(function_name).unwrap(),
            ty_args,
            args,
            &mut GasStatus::new_unmetered(),
        )
        .unwrap_or_else(|e| {
            panic!(
                "Error calling {}.{}: {}",
                module_name,
                function_name,
                e.into_vm_status()
            )
        });
}

fn disable_reconfiguration(session: &mut SessionExt) {
    exec_function_helper(
        session,
        "Reconfiguration",
        "disable_reconfiguration",
        vec![],
        serialize_values(&vec![MoveValue::Signer(AccountAddress::ONE)]),
    );
}

fn enable_reconfiguration(session: &mut SessionExt) {
    exec_function_helper(
        session,
        "Reconfiguration",
        "enable_reconfiguration",
        vec![],
        serialize_values(&vec![MoveValue::Signer(AccountAddress::ONE)]),
    );
}
// TODO: this would be easier than the other way of sending a fork script
//     pub fn exec_script(
//     &mut self,
//     sender: AccountAddress,
//     script: &Script,
// ) -> SerializedReturnValues {
//     let mut temp = vec![sender.to_vec()];
//     temp.extend(convert_txn_args(script.args()));
//     self.0
//         .execute_script(
//             script.code().to_vec(),
//             script.ty_args().to_vec(),
//             temp,
//             &mut UnmeteredGasMeter,
//         )
//         .unwrap()
// }
