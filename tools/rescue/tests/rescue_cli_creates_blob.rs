mod support;

use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::{diem_db_bootstrapper::BootstrapOpts, rescue_tx::RescueTxOpts};

#[tokio::test]
async fn test_valid_genesis() -> anyhow::Result<()> {
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

    println!("1. generate a transaction script which should execute");

    let first_validator_address = env
        .validators()
        .next()
        .unwrap()
        .config()
        .get_peer_id()
        .unwrap();

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let script_path = support::make_script(first_validator_address);

    println!("2. compile the script");

    let r = RescueTxOpts {
        data_path: val_db_path.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: Some(script_path),
        framework_upgrade: false,
        debug_vals: None,
    };
    r.run()?;

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
        db_dir: val_db_path,
        genesis_txn_file: file,
        waypoint_to_verify: Some(wp),
        commit: true,
    };

    let new_w = boot.run()?;

    assert!(wp == new_w, "waypoint mismatch");

    Ok(())
}

#[tokio::test]
async fn test_can_build_gov_rescue_script() -> anyhow::Result<()> {
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

    println!("1. generate a transaction script which should execute");

    let first_validator_address = env
        .validators()
        .next()
        .unwrap()
        .config()
        .get_peer_id()
        .unwrap();

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let script_path = support::make_script(first_validator_address);

    println!("2. compile the script");

    let r = RescueTxOpts {
        data_path: val_db_path,
        blob_path: Some(blob_path.path().to_owned()),
        script_path: Some(script_path),
        framework_upgrade: false,
        debug_vals: None,
    };
    r.run()?;

    let file = blob_path.path().join("rescue.blob");
    assert!(file.exists());

    Ok(())
}

#[tokio::test]
async fn test_valid_waypoint() -> anyhow::Result<()> {
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

    println!("1. generate a transaction script which should execute");

    let remove_first = env
        .validators()
        .next()
        .unwrap()
        .config()
        .get_peer_id()
        .unwrap();

    let blob_path = diem_temppath::TempPath::new();
    blob_path.create_as_dir()?;

    let script_path = support::make_script(remove_first);

    println!("2. compile the script");

    let r = RescueTxOpts {
        data_path: val_db_path.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        script_path: Some(script_path),
        framework_upgrade: false,
        debug_vals: None,
    };
    r.run()?;

    let file = blob_path.path().join("rescue.blob");
    assert!(file.exists());

    println!(
        "3. check we can apply the tx to existing db, and can get a waypoint, don't commit it"
    );

    let boot = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: file,
        waypoint_to_verify: None,
        commit: false,
    };

    let _wp = boot.run()?;

    Ok(())
}
