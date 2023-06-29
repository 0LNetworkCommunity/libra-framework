use anyhow::Error;
use ol_types::legacy_recovery::{self, LegacyRecovery};
use std::path::PathBuf;

/// Make a recovery genesis blob
pub fn parse(recovery_json_path: PathBuf) -> Result<Vec<LegacyRecovery>, Error> {
    Ok(legacy_recovery::read_from_recovery_file(
        &recovery_json_path,
    ))
}


// #[test]
// fn parse_json() {
//     // use crate::convert_types;
//     let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/sample_end_user_single.json");
    
//     let r = parse(p).unwrap();
//     if let Some(acc) = r[0].account {
//         let a = convert_types::convert_account(acc).unwrap();
//         assert!(&a.to_string() == "00000000000000000000000000000000b78ba84a443873f2e324c80f3e4e2334");
//     }

//     if let Some(acc) = r[0].auth_key {
//         let a = convert_types::convert_auth_key(acc).unwrap();
//         dbg!(&hex::encode(&a));
//         assert!(&hex::encode(&a) == "c99e93112458b404daaed49c019c6de5b78ba84a443873f2e324c80f3e4e2334");
//     }
// }