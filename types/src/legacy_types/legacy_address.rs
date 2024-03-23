// NOTE: 0L: Previous to V7 account addresses has 16 bytes: the second half of a 32 byte authentication key. Since V7 the authentication key and address are the same bytes, which means the previous addresses are shorter.
// for compatibility we prepend padded 0s to the legacy address.
// This struct lives here for convenience to use in Genesis where we load
// previous data.

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::account_address::AccountAddress; // NOTE: this is the new type we want to cast into
use hex::FromHex;
use rand::{rngs::OsRng, Rng};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, fmt, str::FromStr};
use move_core_types::move_resource::MoveResource;
use crate::legacy_types::legacy_miner_state::TowerStateResource;
use crate::legacy_types::legacy_recovery::LegacyRecoveryV6;

/// A struct that represents an account address.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub struct LegacyAddress([u8; LegacyAddress::LENGTH]);

impl LegacyAddress {
    pub const fn new(address: [u8; Self::LENGTH]) -> Self {
        Self(address)
    }

    /// The number of bytes in an address.
    pub const LENGTH: usize = 16;

    /// Hex address: 0x0
    pub const ZERO: Self = Self([0u8; Self::LENGTH]);

    pub fn random() -> Self {
        let mut rng = OsRng;
        let buf: [u8; Self::LENGTH] = rng.gen();
        Self(buf)
    }

    pub fn short_str_lossless(&self) -> String {
        let hex_str = hex::encode(self.0).trim_start_matches('0').to_string();
        if hex_str.is_empty() {
            "0".to_string()
        } else {
            hex_str
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }

    pub fn from_hex_literal(literal: &str) -> Result<Self, AccountAddressParseError> {
        if !literal.starts_with("0x") {
            return Err(AccountAddressParseError);
        }

        let hex_len = literal.len() - 2;

        // If the string is too short, pad it
        if hex_len < Self::LENGTH * 2 {
            let mut hex_str = String::with_capacity(Self::LENGTH * 2);
            for _ in 0..Self::LENGTH * 2 - hex_len {
                hex_str.push('0');
            }
            hex_str.push_str(&literal[2..]);
            LegacyAddress::from_hex(hex_str)
        } else {
            LegacyAddress::from_hex(&literal[2..])
        }
    }

    pub fn to_hex_literal(&self) -> String {
        format!("0x{}", self.short_str_lossless())
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| AccountAddressParseError)
            .map(Self)
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| AccountAddressParseError)
            .map(Self)
    }
}

impl AsRef<[u8]> for LegacyAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for LegacyAddress {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for LegacyAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:X}", self)
    }
}

impl fmt::Debug for LegacyAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self)
    }
}

impl fmt::LowerHex for LegacyAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}

impl fmt::UpperHex for LegacyAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

impl From<[u8; LegacyAddress::LENGTH]> for LegacyAddress {
    fn from(bytes: [u8; LegacyAddress::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl TryFrom<&[u8]> for LegacyAddress {
    type Error = AccountAddressParseError;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8]) -> Result<LegacyAddress, AccountAddressParseError> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<Vec<u8>> for LegacyAddress {
    type Error = AccountAddressParseError;

    /// Tries to convert the provided byte buffer into Address.
    fn try_from(bytes: Vec<u8>) -> Result<LegacyAddress, AccountAddressParseError> {
        Self::from_bytes(bytes)
    }
}

impl From<LegacyAddress> for Vec<u8> {
    fn from(addr: LegacyAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<&LegacyAddress> for Vec<u8> {
    fn from(addr: &LegacyAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<LegacyAddress> for [u8; LegacyAddress::LENGTH] {
    fn from(addr: LegacyAddress) -> Self {
        addr.0
    }
}

impl From<&LegacyAddress> for [u8; LegacyAddress::LENGTH] {
    fn from(addr: &LegacyAddress) -> Self {
        addr.0
    }
}

impl From<&LegacyAddress> for String {
    fn from(addr: &LegacyAddress) -> String {
        ::hex::encode(addr.as_ref())
    }
}

impl TryFrom<String> for LegacyAddress {
    type Error = AccountAddressParseError;

    fn try_from(s: String) -> Result<LegacyAddress, AccountAddressParseError> {
        Self::from_hex(s)
    }
}

impl FromStr for LegacyAddress {
    type Err = AccountAddressParseError;

    fn from_str(s: &str) -> Result<Self, AccountAddressParseError> {
        Self::from_hex(s)
    }
}

impl<'de> Deserialize<'de> for LegacyAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            LegacyAddress::from_hex(s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "LegacyAddress")]
            struct Value([u8; LegacyAddress::LENGTH]);

            let value = Value::deserialize(deserializer)?;
            Ok(LegacyAddress::new(value.0))
        }
    }
}

impl Serialize for LegacyAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_hex().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("LegacyAddress", &self.0)
        }
    }
}

impl TryFrom<LegacyAddress> for AccountAddress {
    type Error = anyhow::Error; // Note: two types from legacy and next

    /// Tries to convert legacy address by using string representation hack
    fn try_from(legacy: LegacyAddress) -> Result<AccountAddress, anyhow::Error> {
        let acc_str = legacy.to_hex_literal();
        let new_addr_type = AccountAddress::from_hex_literal(&acc_str)?;
        Ok(new_addr_type)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AccountAddressParseError;

impl fmt::Display for AccountAddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to parse AccoutAddress")
    }
}

impl std::error::Error for AccountAddressParseError {}

#[cfg(test)]
mod tests {
    use super::LegacyAddress;
    use hex::FromHex;
    use std::{
        convert::{AsRef, TryFrom},
        str::FromStr,
    };

    #[test]
    fn test_display_impls() {
        let hex = "ca843279e3427144cead5e4d5999a3d0";
        let upper_hex = "CA843279E3427144CEAD5E4D5999A3D0";

        let address = LegacyAddress::from_hex(hex).unwrap();

        assert_eq!(format!("{}", address), upper_hex);
        assert_eq!(format!("{:?}", address), upper_hex);
        assert_eq!(format!("{:X}", address), upper_hex);
        assert_eq!(format!("{:x}", address), hex);

        assert_eq!(format!("{:#x}", address), format!("0x{}", hex));
        assert_eq!(format!("{:#X}", address), format!("0x{}", upper_hex));
    }

    #[test]
    fn test_short_str_lossless() {
        let address = LegacyAddress::from_hex("00c0f1f95c5b1c5f0eda533eff269000").unwrap();

        assert_eq!(
            address.short_str_lossless(),
            "c0f1f95c5b1c5f0eda533eff269000",
        );
    }

    #[test]
    fn test_short_str_lossless_zero() {
        let address = LegacyAddress::from_hex("00000000000000000000000000000000").unwrap();

        assert_eq!(address.short_str_lossless(), "0");
    }

    #[test]
    fn test_address() {
        let hex = "ca843279e3427144cead5e4d5999a3d0";
        let bytes = Vec::from_hex(hex).expect("You must provide a valid Hex format");

        assert_eq!(
            bytes.len(),
            LegacyAddress::LENGTH,
            "Address {:?} is not {}-bytes long. Addresses must be {} bytes",
            bytes,
            LegacyAddress::LENGTH,
            LegacyAddress::LENGTH,
        );

        let address = LegacyAddress::from_hex(hex).unwrap();

        assert_eq!(address.as_ref().to_vec(), bytes);
    }

    #[test]
    fn test_from_hex_literal() {
        let hex_literal = "0x1";
        let hex = "00000000000000000000000000000001";

        let address_from_literal = LegacyAddress::from_hex_literal(hex_literal).unwrap();
        let address = LegacyAddress::from_hex(hex).unwrap();

        assert_eq!(address_from_literal, address);
        assert_eq!(hex_literal, address.to_hex_literal());

        // Missing '0x'
        LegacyAddress::from_hex_literal(hex).unwrap_err();
        // Too long
        LegacyAddress::from_hex_literal("0x100000000000000000000000000000001").unwrap_err();
    }

    #[test]
    fn test_ref() {
        let address = LegacyAddress::new([1u8; LegacyAddress::LENGTH]);
        let _: &[u8] = address.as_ref();
    }

    #[test]
    fn test_address_from_proto_invalid_length() {
        let bytes = vec![1; 123];
        LegacyAddress::from_bytes(bytes).unwrap_err();
    }

    #[test]
    fn test_deserialize_from_json_value() {
        let address = LegacyAddress::random();
        let json_value = serde_json::to_value(address).expect("serde_json::to_value fail.");
        let address2: LegacyAddress =
            serde_json::from_value(json_value).expect("serde_json::from_value fail.");
        assert_eq!(address, address2)
    }

    #[test]
    fn test_serde_json() {
        let hex = "ca843279e3427144cead5e4d5999a3d0";
        let json_hex = "\"ca843279e3427144cead5e4d5999a3d0\"";

        let address: LegacyAddress = LegacyAddress::from_hex(hex).unwrap();

        let json = serde_json::to_string(&address).unwrap();
        let json_address: LegacyAddress = serde_json::from_str(json_hex).unwrap();

        assert_eq!(json, json_hex);
        assert_eq!(address, json_address);
    }

    #[test]
    fn test_address_from_empty_string() {
        assert!(LegacyAddress::try_from("".to_string()).is_err());
        assert!(LegacyAddress::from_str("").is_err());
    }

    //////// 0L ////////
    #[test]
    fn cast_between_legacy() {
        use diem_types::account_address::AccountAddress;
        let hex = "ca843279e3427144cead5e4d5999a3d0";
        let address: LegacyAddress = LegacyAddress::from_hex(hex).unwrap();
        let old_str = address.to_hex_literal();
        let parsed = AccountAddress::from_hex_literal(&old_str).unwrap();
        let p: AccountAddress = address.try_into().unwrap();
        assert!(parsed == p, "not equal");
    }
}
