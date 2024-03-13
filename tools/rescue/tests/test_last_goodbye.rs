use diem_debugger::DiemDebugger;
use diem_types::{account_config, account_address::AccountAddress};
use diem_vm::move_vm_ext::SessionExt;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use move_core_types::{
    identifier::Identifier,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use move_vm_types::gas::UnmeteredGasMeter;

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
pub async fn test_last_goodbye() -> anyhow::Result<()> {
    // get a clean swarm db with current framework
    let mut smoke = LibraSmoke::new(None).await?;
    let (_, addrs) = smoke.create_accounts(1).await?;
    let bob = *addrs.get(0).unwrap();

    smoke.transfer_from_first_val(bob, 100000).await?;

    let val_one_node = smoke.swarm.validators_mut().next().unwrap();

    val_one_node.stop(); // should safely stop diem-node process, and prevent any DB locks.
    let swarm_db_path = val_one_node.config().storage.dir();

    let debug = DiemDebugger::db(swarm_db_path)?;

    let version = debug.get_latest_version().await?;
    // dbg!(&version);

    let vm_sig = MoveValue::Signer(AccountAddress::ZERO);
    let bob_sig = MoveValue::Signer(bob);


    let _ = debug
        .run_session_at_version(version, |session| {

            execute_fn(session, "last_goodbye", "dont_think_twice_its_alright", vec![&vm_sig, &bob_sig]);

            Ok(())
        })
        .expect("could run session");
    Ok(())
}

fn execute_fn(session: &mut SessionExt, module: &str, function: &str, args: Vec<&MoveValue>) {
    let t = session
        .execute_function_bypass_visibility(
            &ModuleId::new(
                account_config::CORE_CODE_ADDRESS,
                Identifier::new(module).unwrap(),
            ),
            &Identifier::new(function).unwrap(),
            vec![],
            serialize_values(args),
            &mut UnmeteredGasMeter,
        )
        .expect("run function");
    dbg!(&t);
}
