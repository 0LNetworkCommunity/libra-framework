use crate::{diem_db_bootstrapper::BootstrapOpts, session_tools};
use anyhow::Result;
use diem_types::{
    account_address::AccountAddress,
    transaction::{Script, Transaction, WriteSetPayload},
};
use libra_config::validator_registration::{registration_from_operator_yaml, ValCredentials};
use libra_framework::builder::framework_generate_upgrade_proposal::libra_compile_script;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use std::path::{Path, PathBuf};

// #[derive(Parser)]
// /// Create writeset binary files to apply to a db
// pub struct RescueTxOpts {
//     #[clap(short, long)]
//     /// path to the reference db, often $HOME/.libra/data/db
//     pub db_path: PathBuf,
//     #[clap(short, long)]
//     /// directory to read/write or the rescue.blob. Will default to db_path/rescue.blob
//     pub blob_path: Option<PathBuf>,
//     #[clap(short, long)]
//     /// directory to read/write or the rescue.blob
//     pub script_path: Option<PathBuf>,
//     #[clap(long)]
//     /// directory to read/write or the rescue.blob
//     pub framework_upgrade: bool,
//     #[clap(short, long)]
//     /// Replace validator set with these addresses. They must
//     /// already have valid configurations on chain.
//     pub validator_set: Option<Vec<AccountAddress>>,

//     #[clap(long)]
//     /// registers new validators not found on the db, and replaces the validator set.
//     /// Must be in format of operator.yaml (use `libra config validator init``)
//     pub register_vals: Option<Vec<PathBuf>>,
// }

pub fn save_rescue_blob(tx: Transaction, out_dir: &Path) -> Result<PathBuf> {
    let file = out_dir.join("rescue.blob");

    let bytes = bcs::to_bytes(&tx)?;
    std::fs::write(&file, bytes.as_slice())?;
    Ok(file)
}

pub fn run_script_tx(script_path: &Path) -> Result<Transaction> {
    println!("Running script from {:?}", script_path);
    println!("The script must use only functions available installed in the reference db's framework (not what is in the repo source).");
    let (code, _hash) = libra_compile_script(script_path, false)?;

    let wp = WriteSetPayload::Script {
        execute_as: CORE_CODE_ADDRESS,
        script: Script::new(code, vec![], vec![]),
    };

    Ok(Transaction::GenesisTransaction(wp))
}

pub fn upgrade_tx(
    db_path: &Path,
    upgrade_mrb: &Path,
    validator_set: Option<Vec<AccountAddress>>,
) -> Result<Transaction> {
    let cs = session_tools::upgrade_framework_changeset(db_path, validator_set, upgrade_mrb)?;
    Ok(Transaction::GenesisTransaction(WriteSetPayload::Direct(cs)))
}

pub fn register_vals(
    db_path: &Path,
    reg_files: &[PathBuf],
    upgrade_mrb: &Option<PathBuf>,
) -> Result<Transaction> {
    // TODO: replace ValCredentials with OperatorConfiguration
    let registrations: Vec<ValCredentials> = reg_files
        .iter()
        .map(|el| {
            registration_from_operator_yaml(Some(el.to_path_buf()))
                .expect("could parse operator.yaml")
        })
        .collect();

    let cs = session_tools::register_and_replace_validators_changeset(
        db_path,
        registrations,
        upgrade_mrb,
    )?;
    Ok(Transaction::GenesisTransaction(WriteSetPayload::Direct(cs)))
}

pub fn check_rescue_bootstraps(db_path: &Path, blob_path: &Path) -> Result<()> {
    let b = BootstrapOpts {
        db_dir: db_path.to_owned(),
        genesis_txn_file: blob_path.to_owned(),
        waypoint_to_verify: None,
        commit: false,
        info: false,
    };
    if let Some(wp) = b.run()? {
        println!("Rescue tx verified. Bootstrap with waypoint: {:?}", wp);
    }
    Ok(())
}

// impl RescueTxOpts {
//     pub fn run(&self) -> anyhow::Result<PathBuf> {
//         let db_path = self.db_path.clone();

//         // There are three options:
//         // 1. Twin: replace the validator set from config files
//         // 2. Upgrade only: upgrade the framework (maybe the source in reference db is a brick, and can't do the scripts you want).
//         // 3. Run script: the framework in DB is usable, and we need to execute an admin transaction from a .move source

//         // Run a script (Case 3)
//         let gen_tx = if let Some(p) = &self.script_path {
//             let (code, _hash) = libra_compile_script(p, false)?;

//             let wp = WriteSetPayload::Script {
//                 execute_as: CORE_CODE_ADDRESS,
//                 script: Script::new(code, vec![], vec![]),
//             };

//             Transaction::GenesisTransaction(wp)
//         }
//         // Flash the framework (Case 2)
//         else if self.framework_upgrade {
//             let cs =
//                 session_tools::publish_current_framework(&db_path, self.validator_set.to_owned())?;
//             Transaction::GenesisTransaction(WriteSetPayload::Direct(cs))
//         }
//         // Twin Setup (Case 1)
//         else if let Some(reg_files) = self.register_vals.to_owned() {
//             // todo: replace ValCredentials with OperatorConfiguration
//             let registrations: Vec<ValCredentials> = reg_files
//                 .iter()
//                 .map(|el| {
//                     registration_from_operator_yaml(Some(el.to_path_buf()))
//                         .expect("could parse operator.yaml")
//                 })
//                 .collect();

//             let cs = session_tools::twin_testnet(&db_path, registrations)?;
//             Transaction::GenesisTransaction(WriteSetPayload::Direct(cs))
//         } else {
//             anyhow::bail!("no options provided, need a --framework-upgrade or a --script-path");
//         };

//         let mut output = self.blob_path.clone().unwrap_or(db_path);

//         output.push("rescue.blob");

//         let bytes = bcs::to_bytes(&gen_tx)?;
//         std::fs::write(&output, bytes.as_slice())?;

//         Ok(output)
//     }
// }

// #[test]
// fn test_create_blob() -> anyhow::Result<()> {
//     use diem_temppath;
//     use std::path::Path;

//     let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("fixtures")
//         .join("rescue_framework_script");
//     assert!(script_path.exists());

//     let db_root_path = diem_temppath::TempPath::new();
//     db_root_path.create_as_dir()?;
//     let _db = diem_db::DiemDB::new_for_test(db_root_path.path());

//     let blob_path = diem_temppath::TempPath::new();
//     blob_path.create_as_dir()?;

//     let r = RescueTxOpts {
//         db_path: db_root_path.path().to_owned(),
//         blob_path: Some(blob_path.path().to_owned()),
//         script_path: Some(script_path),
//         framework_upgrade: false,
//         validator_set: None,
//         register_vals: None,
//     };
//     r.run()?;

//     assert!(blob_path.path().join("rescue.blob").exists());

//     Ok(())
// }
