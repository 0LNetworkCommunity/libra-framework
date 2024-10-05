// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::version_five::language_storage_v5::StructTagV5;
use crate::version_five::legacy_address_v5::LegacyAddressV5;
use crate::version_five::event_v5::EventHandleV5;

use move_core_types::ident_str;
use move_core_types::{
    identifier::IdentStr,
};
use serde::{Deserialize, Serialize};

use super::legacy_address_v5::LEGACY_CORE_CODE_ADDRESS;
use super::move_resource_v5::{MoveResourceV5, MoveStructTypeV5};


/// The Identifier for the Account module.
pub const DIEM_ACCOUNT_MODULE_IDENTIFIER: &IdentStr = ident_str!("DiemAccount");
/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiemAccountResourceV5 {
    authentication_key: Vec<u8>,
    withdrawal_capability: Option<WithdrawCapabilityResource>,
    key_rotation_capability: Option<KeyRotationCapabilityResource>,
    received_events: EventHandleV5,
    sent_events: EventHandleV5,
    sequence_number: u64,
}

impl DiemAccountResourceV5 {
    /// Constructs an Account resource.
    pub fn new(
        sequence_number: u64,
        authentication_key: Vec<u8>,
        sent_events: EventHandleV5,
        received_events: EventHandleV5,
    ) -> Self {
        DiemAccountResourceV5 {
            authentication_key,
            withdrawal_capability: None,
            key_rotation_capability: None,
            received_events,
            sent_events,
            sequence_number,
        }
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns if this account has delegated its withdrawal capability
    pub fn has_delegated_withdrawal_capability(&self) -> bool {
        self.withdrawal_capability.is_none()
    }

    /// Returns if this account has delegated its key rotation capability
    pub fn has_delegated_key_rotation_capability(&self) -> bool {
        self.key_rotation_capability.is_none()
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &[u8] {
        &self.authentication_key
    }

    /// Return the sent_events handle for the given AccountResource
    pub fn sent_events(&self) -> &EventHandleV5 {
        &self.sent_events
    }

    /// Return the received_events handle for the given AccountResource
    pub fn received_events(&self) -> &EventHandleV5 {
        &self.received_events
    }

    pub fn address(&self) -> LegacyAddressV5 {
        // cast address from
        LegacyAddressV5::from_hex_literal(
            &self
                .sent_events()
                .key()
                .get_creator_address()
                .to_hex_literal(),
        )
        .expect("cannot decode DiemAccount address")
    }

    // for compatibility, the struct tag uses different address format
    pub fn v5_legacy_struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: LEGACY_CORE_CODE_ADDRESS,
            module: ident_str!("DiemAccount").into(),
            name: ident_str!("DiemAccount").into(),
            type_params: vec![],
        }
    }
}

impl MoveStructTypeV5 for DiemAccountResourceV5 {
    const MODULE_NAME: &'static IdentStr = DIEM_ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = DIEM_ACCOUNT_MODULE_IDENTIFIER;
}

impl MoveResourceV5 for DiemAccountResourceV5 {}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawCapabilityResource {
    account_address: LegacyAddressV5,
}

impl MoveStructTypeV5 for WithdrawCapabilityResource {
    const MODULE_NAME: &'static IdentStr = DIEM_ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("WithdrawCapability");
}

impl MoveResourceV5 for WithdrawCapabilityResource {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationCapabilityResource {
    account_address: LegacyAddressV5,
}

impl MoveStructTypeV5 for KeyRotationCapabilityResource {
    const MODULE_NAME: &'static IdentStr = DIEM_ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("KeyRotationCapability");
}

impl MoveResourceV5 for KeyRotationCapabilityResource {}
