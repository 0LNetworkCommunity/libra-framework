// NOTE: 0L: Previous to V7 account addresses has 16 bytes: the second half of a 32 byte authentication key. Since V7 the authentication key and address are the same bytes, which means the previous addresses are shorter.
// for compatibility we prepend padded 0s to the legacy address.
// This struct lives here for convenience to use in Genesis where we load
// previous data.


// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0


use hex::FromHex;
use rand::{rngs::OsRng, Rng};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, fmt, str::FromStr};

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

    /// Generates a random LegacyAddress.
    pub fn random() -> Self {
        let mut rng = OsRng;
        let buf: [u8; Self::LENGTH] = rng.gen();
        Self(buf)
    }

    /// Returns a shortened hexadecimal string representation of the address without leading zeros.
    pub fn short_str_lossless(&self) -> String {
        let hex_str = hex::encode(&self.0).trim_start_matches('0').to_string();
        if hex_str.is_empty() {
            "0".to_string()
        } else {
            hex_str
        }
    }

    /// Converts the address into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Consumes the address and returns the underlying byte array.
    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }

    /// Parses a hexadecimal string with an optional "0x" prefix into a LegacyAddress.
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

    /// Returns the hexadecimal string representation of the address with "0x" prefix.
    pub fn to_hex_literal(&self) -> String {
        format!("0x{}", self.short_str_lossless())
    }

    /// Parses a hexadecimal string into a LegacyAddress.
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| AccountAddressParseError)
            .map(Self)
    }

    /// Returns the hexadecimal string representation of the address.
    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    /// Creates a LegacyAddress from a byte slice.
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, AccountAddressParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| AccountAddressParseError)
            .map(Self)
    }
}

/// Converts the LegacyAddress into a byte slice reference.
impl AsRef<[u8]> for LegacyAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Allows dereferencing the LegacyAddress to a byte array.
impl std::ops::Deref for LegacyAddress {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Formats the LegacyAddress as a string.
impl fmt::Display for LegacyAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:X}", self)
    }
}

/// Formats the LegacyAddress for debugging.
impl fmt::Debug for LegacyAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self)
    }
}


/// Formats the LegacyAddress as a lowercase hexadecimal string.
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

/// Formats the LegacyAddress as an uppercase hexadecimal string.
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

/// Converts a byte array into a LegacyAddress.
impl From<[u8; LegacyAddress::LENGTH]> for LegacyAddress {
    fn from(bytes: [u8; LegacyAddress::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

/// Tries to convert a byte slice into a LegacyAddress.
impl TryFrom<&[u8]> for LegacyAddress {
    type Error = AccountAddressParseError;

    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8]) -> Result<LegacyAddress, AccountAddressParseError> {
        Self::from_bytes(bytes)
    }
}

/// Tries to convert a vector of bytes into a LegacyAddress.
impl TryFrom<Vec<u8>> for LegacyAddress {
    type Error = AccountAddressParseError;

    /// Tries to convert the provided byte buffer into Address.
    fn try_from(bytes: Vec<u8>) -> Result<LegacyAddress, AccountAddressParseError> {
        Self::from_bytes(bytes)
    }
}

/// Converts a LegacyAddress into a vector of bytes.
impl From<LegacyAddress> for Vec<u8> {
    fn from(addr: LegacyAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

/// Converts a reference to a LegacyAddress into a vector of bytes.
impl From<&LegacyAddress> for Vec<u8> {
    fn from(addr: &LegacyAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

/// Converts a LegacyAddress into a byte array.
impl From<LegacyAddress> for [u8; LegacyAddress::LENGTH] {
    fn from(addr: LegacyAddress) -> Self {
        addr.0
    }
}

/// Converts a reference to a LegacyAddress into a byte array.
impl From<&LegacyAddress> for [u8; LegacyAddress::LENGTH] {
    fn from(addr: &LegacyAddress) -> Self {
        addr.0
    }
}

/// Converts a reference to a LegacyAddress into a hexadecimal string.
impl From<&LegacyAddress> for String {
    fn from(addr: &LegacyAddress) -> String {
        ::hex::encode(addr.as_ref())
    }
}

/// Tries to convert a string into a LegacyAddress.
impl TryFrom<String> for LegacyAddress {
    type Error = AccountAddressParseError;

    fn try_from(s: String) -> Result<LegacyAddress, AccountAddressParseError> {
        Self::from_hex(s)
    }
}

/// Tries to convert a string slice into a LegacyAddress.
impl FromStr for LegacyAddress {
    type Err = AccountAddressParseError;

    fn from_str(s: &str) -> Result<Self, AccountAddressParseError> {
        Self::from_hex(s)
    }
}

/// Deserialization implementation for LegacyAddress.
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

/// Serialization implementation for LegacyAddress.
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

#[derive(Clone, Copy, Debug)]
pub struct AccountAddressParseError;

/// Formatting implementation for the AccountAddressParseError.
impl fmt::Display for AccountAddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to parse AccoutAddress")
    }
}

/// Error type for account address parsing failures.
impl std::error::Error for AccountAddressParseError {}

#[cfg(test)]
mod tests {
    use super::LegacyAddress;
    use hex::FromHex;
    use proptest::prelude::*;
    use std::{
        convert::{AsRef, TryFrom},
        str::FromStr,
    };

    /// Test display implementations for LegacyAddress.
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

    /// Test the short string lossless representation for LegacyAddress.
    #[test]
    fn test_short_str_lossless() {
        let address = LegacyAddress::from_hex("00c0f1f95c5b1c5f0eda533eff269000").unwrap();

        assert_eq!(
            address.short_str_lossless(),
            "c0f1f95c5b1c5f0eda533eff269000",
        );
    }

    /// Test the short string lossless representation for a zero address.
    #[test]
    fn test_short_str_lossless_zero() {
        let address = LegacyAddress::from_hex("00000000000000000000000000000000").unwrap();

        assert_eq!(address.short_str_lossless(), "0");
    }

    /// Test address creation from a hexadecimal string.
    #[test]
    fn test_address() {
        let hex = "ca843279e3427144cead5e4d5999a3d0";
        let bytes = Vec::from_hex(hex).expect("You must provide a valid Hex format");

        assert_eq!(
            bytes.len(),
            LegacyAddress::LENGTH as usize,
            "Address {:?} is not {}-bytes long. Addresses must be {} bytes",
            bytes,
            LegacyAddress::LENGTH,
            LegacyAddress::LENGTH,
        );

        let address = LegacyAddress::from_hex(hex).unwrap();

        assert_eq!(address.as_ref().to_vec(), bytes);
    }

    /// Test address creation from a hexadecimal literal string.
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

    /// Test reference conversion for LegacyAddress.
    #[test]
    fn test_ref() {
        let address = LegacyAddress::new([1u8; LegacyAddress::LENGTH]);
        let _: &[u8] = address.as_ref();
    }

    /// Test error handling for invalid byte length during address creation.
    #[test]
    fn test_address_from_proto_invalid_length() {
        let bytes = vec![1; 123];
        LegacyAddress::from_bytes(bytes).unwrap_err();
    }

    /// Test JSON deserialization for LegacyAddress.
    #[test]
    fn test_deserialize_from_json_value() {
        let address = LegacyAddress::random();
        let json_value = serde_json::to_value(address).expect("serde_json::to_value fail.");
        let address2: LegacyAddress =
            serde_json::from_value(json_value).expect("serde_json::from_value fail.");
        assert_eq!(address, address2)
    }

    /// Test JSON serialization for LegacyAddress.
    #[test]
    fn test_serde_json() {
        let hex = "ca843279e3427144cead5e4d5999a3d0";
        let json_hex = "\"ca843279e3427144cead5e4d5999a3d0\"";

        let address = LegacyAddress::from_hex(hex).unwrap();

        let json = serde_json::to_string(&address).unwrap();
        let json_address: LegacyAddress = serde_json::from_str(json_hex).unwrap();

        assert_eq!(json, json_hex);
        assert_eq!(address, json_address);
    }

    /// Test error handling for empty string address creation.
    #[test]
    fn test_address_from_empty_string() {
        assert!(LegacyAddress::try_from("".to_string()).is_err());
        assert!(LegacyAddress::from_str("").is_err());
    }

    proptest! {

        /// Test string roundtrip conversion for LegacyAddress using property-based testing.
        #[test]
        fn test_address_string_roundtrip(addr in any::<LegacyAddress>()) {
            let s = String::from(&addr);
            let addr2 = LegacyAddress::try_from(s).expect("roundtrip to string should work");
            prop_assert_eq!(addr, addr2);
        }

        /// Test protobuf roundtrip conversion for LegacyAddress using property-based testing.
        #[test]
        fn test_address_protobuf_roundtrip(addr in any::<LegacyAddress>()) {
            let bytes = addr.to_vec();
            prop_assert_eq!(bytes.clone(), addr.as_ref());
            let addr2 = LegacyAddress::try_from(&bytes[..]).unwrap();
            prop_assert_eq!(addr, addr2);
        }
    }
}
