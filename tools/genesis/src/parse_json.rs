use libra_types::legacy_types::legacy_recovery::{self, LegacyRecovery};
use std::path::PathBuf;
/// Make a recovery genesis blob
pub fn parse(recovery_json_path: PathBuf) -> anyhow::Result<Vec<LegacyRecovery>> {
    Ok(legacy_recovery::read_from_recovery_file(
        &recovery_json_path,
    ))
}

#[test]
fn parse_json_single() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");

    let r = parse(p).unwrap();
    if let Some(acc) = r[0].account {
        assert!(&acc.to_string() == "6BBF853AA6521DB445E5CBDF3C85E8A0");
    }
}
