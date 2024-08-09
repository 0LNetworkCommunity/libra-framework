// TODO: This is a temporary hack, fix requires changes in Diem repo
// Here we are instantiating the DBtool structs as if it was a command
// line string that was calling it from the stdin.
// NOTE: we do NOT want to call an externa db binary tool here, we are
// just instantiating a rust type at runtime in a gross way.
// we need to initialize the DBTool structs
// however some of the necessary internals are private in the `diem` repo
// so when those structs are make public/vendorized on diem, these hacks
// won't be necessary.

use std::path::Path;

use clap::Parser;

use crate::storage_cli::StorageCli;

/// types of db restore. Note: all of them are necessary for a successful restore.
pub enum RestoreTypes {
    Epoch,
    Snapshot,
    Transaction,
}

pub fn hack_dbtool_init(
    restore_type: RestoreTypes,
    target_db: &Path,
    restore_bundle_dir: &Path,
    manifest: &Path,
    version: u64,
) -> anyhow::Result<StorageCli> {
    // do some checks
    assert!(
        restore_bundle_dir.exists(),
        "backup files does not exist here"
    );
    assert!(target_db.exists(), "target db exists, this will not overwrite but append, you will get in to a weird state, exiting");
    // let manifest_path_base = restore_bundle_dir.join(restore_id);

    let db = target_db.display();
    let fs = restore_bundle_dir.display();
    let mfest = manifest.display();

    let cmd = match restore_type {
        RestoreTypes::Epoch => {
            format!(
                "storage db restore oneoff epoch-ending \n
          --epoch-ending-manifest {mfest} \n
          --target-db-dir {db} \n
          --local-fs-dir {fs} \n
          --target-version {version}"
            )
        }
        RestoreTypes::Snapshot => {
            format!(
                "storage db restore oneoff state-snapshot \n
          --state-manifest {mfest} \n
          --target-db-dir {db} \n
          --local-fs-dir {fs} \n
          --restore-mode default \n
          --target-version {version} \n
          --state-into-version {version}"
            )
        }
        RestoreTypes::Transaction => {
            format!(
                "storage db restore oneoff transaction \n
          --transaction-manifest {mfest} \n
          --target-db-dir {db} \n
          --local-fs-dir {fs} \n
          --target-version {version}",
            )
        }
    };
    dbg!(&cmd);

    let to_vec: Vec<_> = cmd.split_whitespace().collect();
    Ok(StorageCli::try_parse_from(to_vec)?)
}
