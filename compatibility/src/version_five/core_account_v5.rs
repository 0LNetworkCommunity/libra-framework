// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::ident_str;
use move_core_types::{
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

use super::legacy_address_v5::LegacyAddressV5;

pub const CORE_ACCOUNT_MODULE_IDENTIFIER: &IdentStr = ident_str!("Account");

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResourceV5 {
    authentication_key: Vec<u8>,
    sequence_number: u64,
    self_address: LegacyAddressV5,
}

impl AccountResourceV5 {
    /// Constructs an Account resource.
    pub fn new(
        sequence_number: u64,
        authentication_key: Vec<u8>,
        self_address: LegacyAddressV5,
    ) -> Self {
        AccountResourceV5 {
            authentication_key,
            sequence_number,
            self_address,
        }
    }

    /// Return the sequence_number field for the given AccountResourceV5
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the authentication_key field for the given AccountResourceV5
    pub fn authentication_key(&self) -> &[u8] {
        &self.authentication_key
    }

    pub fn address(&self) -> LegacyAddressV5 {
        self.self_address
    }

    //////// 0L /////////
    /// Replace the authkey in place
    pub fn rotate_auth_key(mut self, new_key: Vec<u8>) -> Self {
        self.authentication_key = new_key;
        self
    }

    // pub fn legacy_struct_tag(&self) -> StructTagV5 {

    // }
}

impl MoveStructType for AccountResourceV5 {
    const MODULE_NAME: &'static IdentStr = CORE_ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = CORE_ACCOUNT_MODULE_IDENTIFIER;
}

impl MoveResource for AccountResourceV5 {}
