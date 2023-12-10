
use clap::Parser;
// use diem_logger::prelude::info;
use diem_types::transaction::{Script, Transaction, WriteSetPayload};
use libra_framework::builder::framework_generate_upgrade_proposal::libra_compile_script;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use std::path::PathBuf;

#[derive(Parser)]
/// Create a rescue transaction using the path of the node db
pub struct RescueTxOpts {
    #[clap(short, long)]
    /// directory enclosing the `/db` folder of the node
    pub data_path: PathBuf,
    #[clap(short, long)]
    /// directory to read/write for the rescue.blob. Will default to db_path/rescue.blob
    pub blob_path: Option<PathBuf>,
    #[clap(short, long)]
    /// directory for the governance script to be executed
    pub script_path: PathBuf,
}

impl RescueTxOpts {
    pub async fn run(&self) -> anyhow::Result<PathBuf> {
        let db_path = self.data_path.clone();
        // Deprecation Notice: although still possible, we no longer will try to
        // open a Db with debugger, and create a writeset from a move vm
        // session.

        // There are off chain governance scenarios (fork) for rescuing from halt
        // 1. the framework in DB is usable, and we need to execute an admin
        //    transaction from a .move source
        // 2. we must upgrade the framework because the source in db is a brick.
        // 3. we must execute a transaction that does not exist on the bricked
        // db, but only exists on the upgrade we are executing now.

        // In all cases we want to use the same governance workflow as with
        // onchain governance: craft an admin script.
        // Scenario 1 is easiest. Create a goverance script with the usual
        // process at ./framework/
        // Scenario 2 also uses the same workflow, however the generated code
        // must be adapted so that it is not using the auto-generated
        // diem_governance proposal workflow, and instead is a transaction send
        // by the vm and diem_framework (dual signers)
        // Scenario 3: is most complex. It requires the framework be upgraded
        // first in a separate script. Then a second admin script can be
        // applied. This is because the VM does not have an updated version of
        // the published stdlib until there is a reconfiguration (which issues a
        // number of updates including to cache).

        // Note that if the network is halted at an epoch boundary, there may
        // be additional complications, such as the timestamp of the upgrade
        // taht is being applied is the same as the timestamp of the epoch
        // boundary. This is an intentional check by the reconfiguration.move
        // module. The easiest way out is to advance the TIME by a certain
        // amount.
        // TBD a more straightforward solution.

        println!(
            "attempting to compile governance script at: {}",
            &self.script_path.display()
        );
        // let payload = custom_script(p, None, Some(5));
        let (code, _hash) = libra_compile_script(&self.script_path, false)?;

        let payload = WriteSetPayload::Script {
            execute_as: CORE_CODE_ADDRESS,
            script: Script::new(code, vec![], vec![]),
        };

        println!("governance script encoded");
        let gen_tx = Transaction::GenesisTransaction(payload);

        let bytes = bcs::to_bytes(&gen_tx)?;
        println!("transaction bytes encoded");

        let mut output = self.blob_path.clone().unwrap_or(db_path);
        output.push("rescue.blob");
        std::fs::write(&output, bytes.as_slice())?;
        println!(
            "success: rescue transaction written to: {}",
            output.display()
        );

        Ok(output)
    }
}

#[ignore]
// duplicated test
#[tokio::test]
async fn test_create_blob() -> anyhow::Result<()> {
    use diem_temppath;
    use std::path::Path;

    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("governance_script_template");
    assert!(script_path.exists());

    let db_root_path = diem_temppath::TempPath::new();
    db_root_path.create_as_dir()?;
    let _db = diem_db::DiemDB::new_for_test(db_root_path.path());

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let r = RescueTxOpts {
        data_path: db_root_path.path().to_owned(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: script_path,
    };
    r.run().await?;

    assert!(blob_path.path().join("rescue.blob").exists());

    Ok(())
}
