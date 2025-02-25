use diem_types::account_address::AccountAddress;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypeInfo {
    pub account_address: String,
    #[serde(deserialize_with = "deserialize_string_from_hexstring")]
    pub module_name: String,
    #[serde(deserialize_with = "deserialize_string_from_hexstring")]
    pub struct_name: String,
}

pub fn deserialize_string_from_hexstring<'de, D>(
    deserializer: D,
) -> core::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    Ok(convert_hex(s.clone()).unwrap_or(s))
}

/// Convert the bcs serialized vector<u8> to its original string format
pub fn convert_bcs_hex(typ: String, value: String) -> Option<String> {
    let decoded = hex::decode(value.strip_prefix("0x").unwrap_or(&*value)).ok()?;

    match typ.as_str() {
        "0x1::string::String" => bcs::from_bytes::<String>(decoded.as_slice()),
        "u8" => bcs::from_bytes::<u8>(decoded.as_slice()).map(|e| e.to_string()),
        "u64" => bcs::from_bytes::<u64>(decoded.as_slice()).map(|e| e.to_string()),
        "u128" => bcs::from_bytes::<u128>(decoded.as_slice()).map(|e| e.to_string()),
        "bool" => bcs::from_bytes::<bool>(decoded.as_slice()).map(|e| e.to_string()),
        // 0L NOTE: using AccountAddress here
        "address" => bcs::from_bytes::<AccountAddress>(decoded.as_slice()).map(|e| e.to_string()),
        _ => Ok(value),
    }
    .ok()
}

/// Convert the vector<u8> that is directly generated from b"xxx"
pub fn convert_hex(val: String) -> Option<String> {
    let decoded = hex::decode(val.strip_prefix("0x").unwrap_or(&*val)).ok()?;
    String::from_utf8(decoded).ok()
}
