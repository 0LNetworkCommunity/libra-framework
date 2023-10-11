
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};
use std::path::Path;

#[tokio::test]
async fn test_e2e() -> anyhow::Result<()> {
    let blob_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("basic_genesis.blob");

    let db_root_path = diem_temppath::TempPath::new();
    db_root_path.create_as_dir()?;
    let db = diem_db::DiemDB::new_for_test(db_root_path.path());
    drop(db);

    let boot = BootstrapOpts {
        db_dir: db_root_path.path().to_owned(),
        genesis_txn_file: blob_path,
        waypoint_to_verify: None,
        commit: true,
    };

    boot.run()?;

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("templates")
        .join("governance_script_template");
    assert!(script_path.exists());

    let r = RescueTxOpts {
        data_path: db_root_path.path().to_owned(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: Some(script_path),
        framework_upgrade: false,
    };
    r.run().await?;

    let file = blob_path.path().join("rescue.blob");
    assert!(file.exists());

    let boot = BootstrapOpts {
        db_dir: db_root_path.path().to_owned(),
        genesis_txn_file: file,
        waypoint_to_verify: None,
        commit: true,
    };

    boot.run()?;

    Ok(())
}

// #[test]
// fn test_rescue_db() -> anyhow::Result<()>{
//   use std::path::Path;
//   use diem_temppath;

//   let blob_path = Path::new(env!("CARGO_MANIFEST_DIR"))
//   .join("fixtures")
//   .join("genesis.blob");

//   assert!(blob_path.exists());
//   let db_root_path = diem_temppath::TempPath::new();
//   db_root_path.create_as_dir()?;
//   let db  = diem_db::DiemDB::new_for_test(db_root_path.path());
//   drop(db);

//   let r = BootstrapOpts {
//     db_dir: db_root_path.path().to_owned(),
//     genesis_txn_file: blob_path,
//     waypoint_to_verify: None,
//     commit: true
//   };

//   r.run()?;

//   assert!(db_root_path.path().exists());

//   // run again

//   let blob_path = Path::new(env!("CARGO_MANIFEST_DIR"))
//   .join("fixtures")
//   .join("rescue.blob");

//   let r = BootstrapOpts {
//     db_dir: db_root_path.path().to_owned(),
//     genesis_txn_file: blob_path,
//     waypoint_to_verify: None,
//     commit: true
//   };

//   r.run()?;

//   Ok(())

// }
