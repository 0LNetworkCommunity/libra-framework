use std::path::PathBuf;

pub fn v5_state_manifest_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = p.parent().unwrap();
    let dir = project_root.join("compatibility/fixtures/v5/state_ver_119757649.17a8");
    assert!(
        &dir.exists(),
        "fixtures for backup archive cannot be found at path {}",
        &dir.display()
    );

    dir.to_owned()
}

pub fn v7_state_manifest_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap();
    assert!(&p.exists(), "not at the cargo manifest dir");
    let project_root = p.parent().unwrap();
    assert!(&project_root.exists(), "cannot find project root dir");
    let dir = project_root.join("tools/storage/fixtures/v7/state_epoch_116_ver_38180075.05af");
    assert!(
        &dir.exists(),
        "fixtures for backup archive cannot be found at path {}",
        &dir.display()
    );
    dir
}


pub fn v7_tx_manifest_fixtures_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap();
    assert!(&p.exists(), "not at the cargo manifest dir");
    let project_root = p.parent().unwrap();
    assert!(&project_root.exists(), "cannot find project root dir");
    let dir = project_root.join("tools/storage/fixtures/v7/transaction_38100001-.541f");
    assert!(
        &dir.exists(),
        "fixtures for backup archive cannot be found at path {}",
        &dir.display()
    );
    dir
}
