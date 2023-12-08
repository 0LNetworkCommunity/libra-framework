use crate::framework_payload;
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
    /// directory for the script to be executed
    pub script_path: Option<PathBuf>,
    #[clap(long)]
    /// directory to read/write or the rescue.blob
    pub framework_upgrade: bool,
}

impl RescueTxOpts {
    pub async fn run(&self) -> anyhow::Result<PathBuf> {
        let db_path = self.data_path.clone();

        // There are two options:
        // 1. upgrade the framework because the source in db is a brick.
        // 2. the framework in DB is usable, and we need to execute an admin
        //    transaction from a .move source

        let gen_tx = if let Some(p) = &self.script_path {
            println!("attempting to compile governance script at: {}", p.display());
            // let payload = custom_script(p, None, Some(5));
            let (code, _hash) = libra_compile_script(p, false)?;

            let wp = WriteSetPayload::Script {
                execute_as: CORE_CODE_ADDRESS,
                script: Script::new(code, vec![], vec![]),
            };
            // info!("governance script encoded");
            Transaction::GenesisTransaction(wp)
        } else if self.framework_upgrade {
            let payload = framework_payload::stlib_payload(db_path.clone()).await?;
            // warn!("stdlib writeset encoded");
            println!("stdlib writeset encoded");
            Transaction::GenesisTransaction(payload)
        } else {
            anyhow::bail!("no options provided, need a --framework-upgrade or a --script-path");
        };

        let bytes = bcs::to_bytes(&gen_tx)?;
        println!("transaction bytes encoded");

        let mut output = self.blob_path.clone().unwrap_or(db_path);
        output.push("rescue.blob");
        std::fs::write(&output, bytes.as_slice())?;
        println!("SUCCESS: rescue transaction written to: {}", output.display());

        Ok(output)
    }
}

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
        script_path: Some(script_path),
        framework_upgrade: false,
    };
    r.run().await?;

    assert!(blob_path.path().join("rescue.blob").exists());

    // db_root_path.path()

    Ok(())
}
