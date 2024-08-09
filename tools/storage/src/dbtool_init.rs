//! refactoring the internals of Vendor DBTool::Oneoff
//! source: <diem> storage/db-tool/src/restore.rs
//!
use crate::restore_bundle::RestoreBundle;

use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use diem_backup_cli::{
    backup_types::{
        epoch_ending::restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::{TransactionRestoreController, TransactionRestoreOpt},
    },
    storage::{local_fs::LocalFs, BackupStorage},
    utils::{GlobalRestoreOptions, RestoreRunMode, TrustedWaypointOpt},
};
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::{state_restore::StateSnapshotRestoreMode, DiemDB, GetRestoreHandler};
use diem_executor_types::VerifyExecutionMode;

/// types of db restore. Note: all of them are necessary for a successful restore.
pub enum RestoreTypes {
    Epoch,
    Snapshot,
    Transaction,
}

pub fn get_backup_storage(local_fs_dir: PathBuf) -> Result<Arc<dyn BackupStorage>> {
    Ok(Arc::new(LocalFs::new(local_fs_dir)))
}

pub fn get_global_db_opts(
    db_dir: PathBuf,
    bundle: &RestoreBundle,
) -> anyhow::Result<GlobalRestoreOptions> {
    let restore_handler = Arc::new(DiemDB::open_kv_only(
        db_dir,
        false,                       /* read_only */
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner config */
        RocksdbConfigs::default(),
        false,
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    )?)
    .get_restore_handler();

    let run_mode = RestoreRunMode::Restore { restore_handler };

    let twp = TrustedWaypointOpt {
        trust_waypoint: vec![bundle.waypoint.expect("no waypoint")],
    };

    Ok(GlobalRestoreOptions {
        target_version: bundle.version,
        trusted_waypoints: Arc::new(twp.verify().expect("cannot verify waypoint")),
        run_mode: Arc::new(run_mode),
        concurrent_downloads: num_cpus::get(),
        replay_concurrency_level: 4,
    })
}

pub async fn run_restore(
    rtype: RestoreTypes,
    db_path: PathBuf,
    bundle: &RestoreBundle,
) -> anyhow::Result<()> {
    let global = get_global_db_opts(db_path.clone(), &bundle)?;

    let storage = get_backup_storage(bundle.restore_bundle_dir.to_path_buf())?;

    match rtype {
        RestoreTypes::Epoch => {
            EpochEndingRestoreController::new(
                EpochEndingRestoreOpt {
                    manifest_handle: bundle.epoch_manifest.to_str().unwrap().to_string(),
                },
                global,
                storage,
            )
            .run(None)
            .await?;
        }
        RestoreTypes::Snapshot => {
            StateSnapshotRestoreController::new(
                StateSnapshotRestoreOpt {
                    manifest_handle: bundle.snapshot_manifest.to_str().unwrap().to_string(),
                    version: bundle.version,
                    validate_modules: false,
                    restore_mode: StateSnapshotRestoreMode::Default,
                },
                global,
                storage,
                None, /* epoch_history */
            )
            .run()
            .await?;
        }
        RestoreTypes::Transaction => {
            TransactionRestoreController::new(
                TransactionRestoreOpt {
                    manifest_handle: bundle.transaction_manifest.to_str().unwrap().to_string(),
                    replay_from_version: None,
                    kv_only_replay: None,
                },
                global,
                storage,
                None, /* epoch_history */
                VerifyExecutionMode::NoVerify,
            )
            .run()
            .await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_restore() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut b = RestoreBundle::new(dir.join("fixtures/v7"));
    b.load().unwrap();
    let db_temp = diem_temppath::TempPath::new();
    db_temp.create_as_dir().unwrap();
    run_restore(RestoreTypes::Epoch, db_temp.path().to_owned(), &b)
        .await
        .unwrap();
    run_restore(RestoreTypes::Snapshot, db_temp.path().to_owned(), &b)
        .await
        .unwrap();
    run_restore(RestoreTypes::Transaction, db_temp.path().to_owned(), &b)
        .await
        .unwrap();
}
