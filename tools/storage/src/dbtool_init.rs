use anyhow::Result;
use diem_backup_cli::{
    backup_types::epoch_ending::restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{GlobalRestoreOptions, RestoreRunMode, TrustedWaypointOpt},
};
use std::{path::PathBuf, sync::Arc};

pub fn init_storage(local_fs_dir: PathBuf) -> Result<Arc<dyn BackupStorage>> {
    Ok(Arc::new(LocalFs::new(local_fs_dir)))
}

pub fn epoch_restore_opts(manifest_path: &str) -> EpochEndingRestoreOpt {
    EpochEndingRestoreOpt {
        manifest_handle: manifest_path.to_string(),
    }
}

pub fn trusted_waypoints(wp_str: &str) -> TrustedWaypointOpt {
    let waypoint = wp_str.parse().expect("cannot parse waypoint");
    TrustedWaypointOpt {
        trust_waypoint: vec![waypoint],
    }
}

pub fn epoch_controller(
    new_db_path: PathBuf,
    manifest_path: PathBuf,
    waypoint_str: &str,
) -> EpochEndingRestoreController {
    let twp = trusted_waypoints(waypoint_str)
        .verify()
        .expect("could not format waypoint");

    let epoch_restore_opts = epoch_restore_opts(manifest_path.to_str().expect("expect path str"));

    let global_restore_opts = GlobalRestoreOptions {
        run_mode: Arc::new(RestoreRunMode::Verify),
        target_version: 0,
        concurrent_downloads: 4,
        trusted_waypoints: Arc::new(twp),
        replay_concurrency_level: 0,
    };
    let db = init_storage(new_db_path).expect("could not init storage");

    EpochEndingRestoreController::new(epoch_restore_opts, global_restore_opts, db)
}

#[tokio::test]
async fn try_read_manifest() {
    use diem_temppath::TempPath;

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let temp = TempPath::new();
    temp.create_as_dir().unwrap();

    let db_path = temp.path();
    assert!(&db_path.exists());

    let waypoint_str = "116:b4c9918ddb62469cc3e7e7b2a01b43aeac803470913b3a89afdcc44078df8d8a";
    let manifest_path = crate_dir.join("fixtures/v7/epoch_ending_116-.be9b/epoch_ending.manifest");
    dbg!(&manifest_path);
    assert!(&manifest_path.exists());

    let controller = epoch_controller(db_path.to_owned(), manifest_path, waypoint_str);
    let res = controller.run(None).await.expect("run failed");
    dbg!(&res);
}
