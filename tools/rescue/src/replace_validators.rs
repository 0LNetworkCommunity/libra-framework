use diem_types::transaction::{Transaction, WriteSetPayload};
use libra_config::validator_registration::ValCredentials;
use std::path::{Path, PathBuf};

use crate::session_tools;

// TODO: replace with calling the rescue_cli directly.
/// Make a rescue blob with the given credentials
/// credentials are usually saved by the libra-config tool
/// as a operator.yaml
pub async fn replace_validators_blob(
    db_path: &Path,
    creds: Vec<ValCredentials>,
    output_dir: &Path,
) -> anyhow::Result<PathBuf> {
    println!("run session to create validator onboarding tx (replace_validators_rescue.blob)");

    let cs = session_tools::register_and_replace_validators_changeset(db_path, creds, &None)?;

    let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));
    let out = output_dir.join("replace_validators_rescue.blob");

    let bytes = bcs::to_bytes(&gen_tx)?;
    std::fs::write(&out, bytes.as_slice())?;
    Ok(out)
}
