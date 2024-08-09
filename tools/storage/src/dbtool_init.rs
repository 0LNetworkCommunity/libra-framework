use crate::restore_bundle::RestoreBundle;
use anyhow::Result;
use diem_backup_cli::{
    backup_types::epoch_ending::restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{GlobalRestoreOptions, RestoreRunMode, TrustedWaypointOpt},
};
use diem_config::config::{RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD, NO_OP_STORAGE_PRUNER_CONFIG};
use diem_db::{DiemDB, GetRestoreHandler};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// types of db restore. Note: all of them are necessary for a successful restore.
pub enum RestoreTypes {
    Epoch,
    Snapshot,
    Transaction,
}

pub fn init_storage(local_fs_dir: PathBuf) -> Result<Arc<dyn BackupStorage>> {
    Ok(Arc::new(LocalFs::new(local_fs_dir)))
}

// pub fn get_restorerun_restore() -> RestoreRunMode {
//   get_restore_handler()
// }

pub fn get_global_db_opts(db_dir: PathBuf, bundle: &RestoreBundle) -> anyhow::Result<GlobalRestoreOptions> {
    // GlobalRestoreOpt::default().try_into().unwrap()
    // for restore, we can always start state store with empty buffered_state since we will restore
    let restore_handler = Arc::new(DiemDB::open_kv_only(
        db_dir,
        false,                       /* read_only */
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner config */
        RocksdbConfigs::default(),                        // opt.rocksdb_opt.clone().into(),
        false,
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    )?)
    .get_restore_handler();

     let twp  = TrustedWaypointOpt {
        trust_waypoint: vec![bundle.waypoint.expect("no waypoint")],
    };

    let run_mode = RestoreRunMode::Restore { restore_handler };
    Ok(GlobalRestoreOptions {
        target_version: bundle.version,
        trusted_waypoints: Arc::new(twp.verify().expect("cannot verify waypoint")),
        run_mode: Arc::new(run_mode),
        concurrent_downloads: 0,
        replay_concurrency_level: 4,
    })
}

pub fn epoch_restore_opts(manifest_path: &Path) -> EpochEndingRestoreOpt {
    EpochEndingRestoreOpt {
        manifest_handle: manifest_path.to_str().unwrap().to_owned(),
    }
}

pub fn trusted_waypoints(wp_str: &str) -> TrustedWaypointOpt {
    let waypoint = wp_str.parse().expect("cannot parse waypoint");
    TrustedWaypointOpt {
        trust_waypoint: vec![waypoint],
    }
}

// pub fn epoch_controller(
//     new_db_path: PathBuf,
//     manifest_path: PathBuf,
//     waypoint_str: &str,
// ) -> EpochEndingRestoreController {
//     let twp = trusted_waypoints(waypoint_str)
//         .verify()
//         .expect("could not format waypoint");

//     let epoch_restore_opts = epoch_restore_opts(manifest_path);

//     let global_restore_opts = GlobalRestoreOptions {
//         run_mode: Arc::new(RestoreRunMode::Verify),
//         target_version: 0,
//         concurrent_downloads: 4,
//         trusted_waypoints: Arc::new(twp),
//         replay_concurrency_level: 0,
//     };
//     let db = init_storage(new_db_path).expect("could not init storage");

//     EpochEndingRestoreController::new(epoch_restore_opts, global_restore_opts, db)
// }

pub async fn run_restore(
    rtype: RestoreTypes,
    db_path: PathBuf,
    bundle: RestoreBundle,
) -> anyhow::Result<()> {
    // let epoch_restore_opts = epoch_restore_opts(manifest_path.to_str().expect("expect path str"));
    let global = get_global_db_opts(db_path.clone(), &bundle)?;
    let storage = init_storage(db_path)?;

    match rtype {
        RestoreTypes::Epoch => {
            EpochEndingRestoreController::new(
                epoch_restore_opts(&bundle.epoch_manifest),
                global,
                storage,
            )
            .run(None)
            .await?;
        }
        // RestoreTypes::Snapshot => {
        //     StateSnapshotRestoreController::new(
        //         opt,
        //         global.try_into()?,
        //         storage.init_storage().await?,
        //         None, /* epoch_history */
        //     )
        //     .run()
        //     .await?;
        // },
        // RestoreTypes::Transaction => {
        //     TransactionRestoreController::new(
        //         opt,
        //         global,
        //         storage,,
        //         None, /* epoch_history */
        //         VerifyExecutionMode::NoVerify,
        //     )
        //     .run()
        //     .await?;
        // },
        _ => todo!(),
    }
    Ok(())
}

// #[tokio::test]
// async fn try_read_manifest() {
//     use diem_temppath::TempPath;

//     let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

//     let temp = TempPath::new();
//     temp.create_as_dir().unwrap();

//     let db_path = temp.path();
//     assert!(&db_path.exists());

//     let waypoint_str = "116:b4c9918ddb62469cc3e7e7b2a01b43aeac803470913b3a89afdcc44078df8d8a";
//     let manifest_path = crate_dir.join("fixtures/v7/epoch_ending_116-.be9b/epoch_ending.manifest");
//     dbg!(&manifest_path);
//     assert!(&manifest_path.exists());

//     let controller = epoch_controller(db_path.to_owned(), manifest_path, waypoint_str);
//     let res = controller.run(None).await.expect("run failed");
//     dbg!(&res);
// }
