use crate::{session_tools, user_file::UserBlob};
use clap::Parser;
use diem_types::transaction::{Script, Transaction, WriteSetPayload};
use libra_framework::builder::framework_generate_upgrade_proposal::libra_compile_script;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use std::path::PathBuf;

#[derive(Parser)]
/// Apply transactions to a DB at rest (fork the chain)
pub struct RescueTxOpts {
    #[clap(short, long)]
    /// directory enclosing the `/db` folder of the node
    pub db_dir: PathBuf,
    #[clap(short, long)]
    /// directory to read/write or the rescue.blob. Will default to db_path/rescue.blob
    pub blob_path: Option<PathBuf>,
    #[clap(short, long)]
    /// directory to read/write or the rescue.blob
    pub script_path: Option<PathBuf>,
    #[clap(long)]
    /// directory to read/write or the rescue.blob
    pub framework_upgrade: bool,
    #[clap(long)]
    /// optional, JSON file with list of new validators. Must already have on-chain configurations
    #[clap(short, long)]
    pub validators_file: Option<PathBuf>,
}

impl RescueTxOpts {
    pub fn run(&self) -> anyhow::Result<PathBuf> {
        let db_path = self.db_dir.clone();

        // There are two options:
        // 1. upgrade the framework because the source in db is a brick.
        // 2. the framework in DB is usable, and we need to execute an admin
        //    transaction from a .move source

        let gen_tx = if let Some(p) = &self.script_path {
            // let payload = custom_script(p, None, Some(5));
            let (code, _hash) = libra_compile_script(p, false)?;

            let wp = WriteSetPayload::Script {
                execute_as: CORE_CODE_ADDRESS,
                script: Script::new(code, vec![], vec![]),
            };

            Transaction::GenesisTransaction(wp)
        } else if self.framework_upgrade {
            let vals = UserBlob::get_vals(self.validators_file.clone());
            let cs = session_tools::publish_current_framework(&db_path, vals)?;
            Transaction::GenesisTransaction(WriteSetPayload::Direct(cs))
        } else {
            anyhow::bail!("no options provided, need a --framework-upgrade or a --script-path");
        };

        let mut output = self.blob_path.clone().unwrap_or(db_path);

        output.push("rescue.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&output, bytes.as_slice())?;

        Ok(output)
    }
}

#[test]
fn test_create_blob() -> anyhow::Result<()> {
    use diem_temppath;
    use std::path::Path;

    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("rescue_framework_script");
    assert!(script_path.exists());

    let db_root_path = diem_temppath::TempPath::new();
    db_root_path.create_as_dir()?;
    let _db = diem_db::DiemDB::new_for_test(db_root_path.path());

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let r = RescueTxOpts {
        db_dir: db_root_path.path().to_owned(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: Some(script_path),
        framework_upgrade: false,
        validators_file: None,
    };
    r.run()?;

    assert!(blob_path.path().join("rescue.blob").exists());

    // db_root_path.path()

    Ok(())
}
