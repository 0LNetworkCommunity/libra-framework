
// TODO: This is a temporary hack, fix requires changes in Diem repo
// Here we are instantiating the DBtool structs as if it was a command
// line string that was calling it from the stdin.
// NOTE: we do NOT want to call an externa db binary tool here, we are
// just instantiating a rust type at runtime in a gross way.
// we need to initialize the DBTool structs
// however some of the necessary internals are private in the `diem` repo
// so when those structs are make public/vendorized on diem, these hacks
// won't be necessary.



enum RestoreTypes {
  Epoch,
  Snapshot,
  Transaction,
}

pub fn hack_dbtool_init(restore_type: RestoreTypes,  target_db: &Path, restore_files: &Path, restore_id: &str, version: u64) {
    // do some checks
    assert!(restore_files.exists(), "backup files does not exist here");
    let manifest_path_base = restore_file.join(restore_id);
    assert!(manifest_path.exists(), "restore manifest does not exist");
    assert!(target_db.exists(), "target db exists, this will not overwrite but append, you will get in to a weird state, exiting");

    let db = target_db.display(),
    let fs = restore_files.display(),

   let cmd =  match restore_type {
      Epoch => {
        let manifest = manifest_path_base.join("epoch-ending.manifest").display();
              format!("storage db restore oneoff epoch-ending \
              --epoch-ending-manifest {manifest} \
              --target-db-dir {db} \
              --local-fs-dir {fs} \
              --target-version {version}",
      );

      }
      _ -> "todo()"
    }


    let to_vec: Vec<_> = cmd.split_whitespace().collect();
    let s = StorageCli::try_parse_from(to_vec)?;
}
