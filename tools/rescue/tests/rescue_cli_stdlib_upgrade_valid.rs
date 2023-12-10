mod support;

use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::diem_db_bootstrapper::BootstrapOpts;
use rescue::rescue_tx::RescueTxOpts;
use std::path::PathBuf;

#[tokio::test]
async fn test_framwork_upgrade_writeset() -> anyhow::Result<()> {
    println!("0. create a valid test database from smoke-tests");
    let mut s = LibraSmoke::new(Some(3))
        .await
        .expect("could not start libra smoke");

    let env = &mut s.swarm;

    let val_db_path = env.validators().next().unwrap().config().storage.dir();
    assert!(val_db_path.exists());

    for node in env.validators_mut() {
        node.stop();
    }

    println!("1. generate framework upgrade writeset which should execute");
    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let rescue_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script_path = rescue_script
        .join("fixtures")
        .join("rescue_framework_script");

    let r = RescueTxOpts {
        data_path: val_db_path.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path,
    };
    r.run().await?;

    let file = blob_path.path().join("rescue.blob");
    assert!(file.exists());

    println!(
        "3. check we can apply the tx to existing db, and can get a waypoint, don't commit it"
    );

    let boot = BootstrapOpts {
        db_dir: val_db_path.clone(),
        genesis_txn_file: file.clone(),
        waypoint_to_verify: None,
        commit: false,
    };

    let wp = boot.run()?;

    println!("4. with the known waypoint confirm it, and apply the tx");
    let boot = BootstrapOpts {
        db_dir: val_db_path.clone(),
        genesis_txn_file: file,
        waypoint_to_verify: Some(wp),
        commit: true,
    };

    let new_w = boot.run()?;

    assert!(wp == new_w, "waypoint mismatch");

    Ok(())
}
