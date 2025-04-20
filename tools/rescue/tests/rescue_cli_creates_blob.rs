use libra_rescue::{
    cli_bootstrapper::BootstrapOpts,
    cli_main::{RescueCli, Sub, RUN_SCRIPT_BLOB},
    test_support,
};
use libra_smoke_tests::libra_smoke::LibraSmoke;

#[tokio::test]
async fn test_valid_genesis() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new(Some(3), None)
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

    let script_path = test_support::make_script(first_validator_address);

    println!("2. compile the script");

    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        command: Sub::RunScript {
            script_path: Some(script_path),
        },
    };

    r.run()?;

    let file = blob_path.path().join(RUN_SCRIPT_BLOB);
    assert!(file.exists());

    // hack, adding sleep here since we get db lock issue in CI.
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!(
        "3. check we can apply the tx to existing db, and can get a waypoint, don't commit it"
    );

    let boot = BootstrapOpts {
        db_dir: val_db_path.clone(),
        genesis_txn_file: file.clone(),
        waypoint_to_verify: None,
        commit: false,
        update_node_config: None,
        info: false,
    };

    let wp = boot.run()?;

    // hack, adding sleep here since we get db lock issue in CI.
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("4. with the known waypoint confirm it, and apply the tx");
    let boot = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: file,
        waypoint_to_verify: wp,
        commit: true,
        update_node_config: None,
        info: false,
    };

    let new_w = boot.run()?;

    assert_eq!(wp, new_w, "waypoint mismatch");

    Ok(())
}

#[tokio::test]
async fn test_can_build_gov_rescue_script() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new(Some(3), None)
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

    let script_path = test_support::make_script(first_validator_address);

    println!("2. compile the script");

    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        command: Sub::RunScript {
            script_path: Some(script_path),
        },
    };

    r.run()?;

    let file = blob_path.path().join(RUN_SCRIPT_BLOB);
    assert!(file.exists());

    Ok(())
}

#[tokio::test]
async fn test_valid_waypoint() -> anyhow::Result<()> {
    let mut s = LibraSmoke::new(Some(3), None)
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

    let script_path = test_support::make_script(remove_first);

    println!("2. compile the script");

    let r = RescueCli {
        db_path: val_db_path.clone(),
        blob_path: Some(blob_path.path().to_owned()),
        command: Sub::RunScript {
            script_path: Some(script_path),
        },
    };

    r.run()?;

    let file = blob_path.path().join(RUN_SCRIPT_BLOB);
    assert!(file.exists());

    println!(
        "3. check we can apply the tx to existing db, and can get a waypoint, don't commit it"
    );

    let boot = BootstrapOpts {
        db_dir: val_db_path,
        genesis_txn_file: file,
        waypoint_to_verify: None,
        commit: false,
        update_node_config: None,
        info: false,
    };

    let _wp = boot.run()?;

    Ok(())
}
