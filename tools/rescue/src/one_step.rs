use std::{path::Path, time::Duration};
use diem_types::waypoint::Waypoint;
use crate::diem_db_bootstrapper::BootstrapOpts;

/// helper to apply a single step upgrade to a db
/// usually for internal testing purposes (e.g. twin)
pub fn apply_rescue_on_db(
        db_to_change_path: &Path,
        rescue_blob: &Path,
    ) -> anyhow::Result<Waypoint> {
        let mut waypoint: Option<Waypoint> = None;
        let bootstrap = BootstrapOpts {
            db_dir: db_to_change_path.to_owned(),
            genesis_txn_file: rescue_blob.to_owned(),
            waypoint_to_verify: None,
            commit: false, // NOT APPLYING THE TX
            info: false,
        };

        let waypoint_to_check = bootstrap.run()?.expect("could not get waypoint");

        // give time for any IO to finish
        std::thread::sleep(Duration::from_secs(1));

        let bootstrap = BootstrapOpts {
            db_dir: db_to_change_path.to_owned(),
            genesis_txn_file: rescue_blob.to_owned(),
            waypoint_to_verify: Some(waypoint_to_check),
            commit: true, // APPLY THE TX
            info: false,
        };

        let waypoint_post = bootstrap.run()?.expect("could not get waypoint");
        assert!(
            waypoint_to_check == waypoint_post,
            "waypoints are not equal"
        );
        if let Some(w) = waypoint {
            assert!(
                waypoint_to_check == w,
                "waypoints are not equal between nodes"
            );
        }
        waypoint = Some(waypoint_to_check);
        // }
        match waypoint {
            Some(w) => Ok(w),
            None => bail!("cannot generate consistent waypoint."),
        }
    }
