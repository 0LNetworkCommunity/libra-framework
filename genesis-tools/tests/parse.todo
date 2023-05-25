use std::path::PathBuf;
use genesis_tools::{parse_json, convert_types};

#[test]

fn parse_json() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_end_user_single.json");
    
    // dbg!(&p);
    let r = parse_json::parse(p).unwrap();
    // dbg!(&r);
    if let Some(acc) = r[0].account {
        let a = convert_types::convert_account(acc).unwrap();
        // dbg!(&a);
        assert!(&a.to_string() == "00000000000000000000000000000000b78ba84a443873f2e324c80f3e4e2334");
    }

    if let Some(acc) = r[0].auth_key {
        let a = convert_types::convert_auth_key(acc).unwrap();
        dbg!(&hex::encode(&a));
        assert!(&hex::encode(&a) == "c99e93112458b404daaed49c019c6de5b78ba84a443873f2e324c80f3e4e2334");
    }
}