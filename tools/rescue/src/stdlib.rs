use anyhow;
use diem_types::transaction::ChangeSet;
use std::path::PathBuf;

pub fn stlib_changeset(path: PathBuf) -> anyhow::Result<ChangeSet> {
    let db = DiemDebugger::db(path)?;

    // publish the agreed stdlib
    let new_stdlib = libra_framework::release::modules();

    let v = db.get_latest_version()?;
    db.run_session_at_version(v, None, |session| {
        let mut gas_status = GasStatus::new_unmetered();

        for module in new_stdlib {
            let mut bytes = vec![];
            module.serialize(&mut bytes).unwrap();

            session
                .revise_module(
                    bytes,
                    account_config::CORE_CODE_ADDRESS,
                    &mut gas_status,
                )
                .unwrap()
        }
        Ok(())
    })
}