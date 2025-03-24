use diem_types::transaction::{Transaction, WriteSetPayload};
use libra_config::validator_registration::ValCredentials;
use libra_rescue::cli_main::REPLACE_VALIDATORS_BLOB;
use std::path::{Path, PathBuf};

use libra_rescue::{session_tools, transaction_factory};

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
    let out = output_dir.join(REPLACE_VALIDATORS_BLOB);
    transaction_factory::save_rescue_blob(gen_tx, &out)?;
    Ok(out)
}
