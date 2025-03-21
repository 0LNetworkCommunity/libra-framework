use std::path::{Path, PathBuf};
use diem_types::transaction::{Transaction, WriteSetPayload};
use libra_config::validator_registration::ValCredentials;

use crate::session_tools::{self, libra_run_session, session_add_validators};

/// Make a rescue blob with the given credentials
/// credentials are usually saved by the libra-config tool
/// as a operator.yaml
pub async fn replace_validators_blob(
    db_path: &Path,
    creds: Vec<ValCredentials>,
) -> anyhow::Result<PathBuf> {
    println!("run session to create validator onboarding tx (replace_validators_rescue.blob)");
    let vmc = libra_run_session(
        db_path.to_path_buf(),
        |session| session_add_validators(session, creds, false),
        None,
        None,
    )?;

    let cs = session_tools::unpack_changeset(vmc)?;

    let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));
    let out = db_path.join("replace_validators_rescue.blob");

    let bytes = bcs::to_bytes(&gen_tx)?;
    std::fs::write(&out, bytes.as_slice())?;
    Ok(out)
}
