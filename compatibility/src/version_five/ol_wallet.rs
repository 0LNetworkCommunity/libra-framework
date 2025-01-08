use crate::version_five::{language_storage_v5::StructTagV5, move_resource_v5::MoveStructTypeV5};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

use super::{
    language_storage_v5::CORE_CODE_ADDRESS, legacy_address_v5::LegacyAddressV5,
    move_resource_v5::MoveResourceV5,
};

// NOTE: these are legacy structs for v5

/// Struct that represents a CommunityWallet resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityWalletsResourceLegacyV5 {
    /// List
    pub list: Vec<LegacyAddressV5>,
}

impl MoveStructTypeV5 for CommunityWalletsResourceLegacyV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("Wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CommunityWalletList");
}

impl CommunityWalletsResourceLegacyV5 {
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: CommunityWalletsResourceLegacyV5::module_identifier(),
            name: CommunityWalletsResourceLegacyV5::struct_identifier(),
            type_params: vec![],
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletResourceV5 {
    pub unlocked: u64,
    pub transferred: u64,
}

impl MoveStructTypeV5 for SlowWalletResourceV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWallet");
}

impl MoveResourceV5 for SlowWalletResourceV5 {}

impl SlowWalletResourceV5 {
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: SlowWalletResourceV5::module_identifier(),
            name: SlowWalletResourceV5::struct_identifier(),
            type_params: vec![],
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletListResourceV5 {
    pub list: Vec<LegacyAddressV5>,
}

impl MoveStructTypeV5 for SlowWalletListResourceV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletList");
}

impl SlowWalletListResourceV5 {
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: SlowWalletListResourceV5::module_identifier(),
            name: SlowWalletListResourceV5::struct_identifier(),
            type_params: vec![],
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
