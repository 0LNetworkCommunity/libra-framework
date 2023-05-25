use anyhow::Error;
use ol_types::legacy_recovery::{self, LegacyRecovery};
use std::path::PathBuf;

/// Make a recovery genesis blob
pub fn parse(recovery_json_path: PathBuf) -> Result<Vec<LegacyRecovery>, Error> {
    Ok(legacy_recovery::read_from_recovery_file(
        &recovery_json_path,
    ))
}
