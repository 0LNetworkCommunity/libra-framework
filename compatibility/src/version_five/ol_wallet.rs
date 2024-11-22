use crate::version_five::{language_storage_v5::StructTagV5, move_resource_v5::MoveStructTypeV5};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

use super::{language_storage_v5::CORE_CODE_ADDRESS, legacy_address_v5::LegacyAddressV5};

// NOTE: these are legacy structs for v5

/// Struct that represents a CommunityWallet resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityWalletsResourceLegacy {
    /// List
    pub list: Vec<LegacyAddressV5>,
}

impl MoveStructTypeV5 for CommunityWalletsResourceLegacy {
    const MODULE_NAME: &'static IdentStr = ident_str!("Wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CommunityWalletList");
}

impl CommunityWalletsResourceLegacy {
    ///
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: CommunityWalletsResourceLegacy::module_identifier(),
            name: CommunityWalletsResourceLegacy::struct_identifier(),
            type_params: vec![],
        }
    }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletResource {
    ///
    pub unlocked: u64,
    ///
    pub transferred: u64,
}

impl MoveStructTypeV5 for SlowWalletResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWallet");
}

impl SlowWalletResource {
    ///
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: SlowWalletResource::module_identifier(),
            name: SlowWalletResource::struct_identifier(),
            type_params: vec![],
        }
    }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletListResource {
    ///
    pub list: Vec<LegacyAddressV5>,
}

impl MoveStructTypeV5 for SlowWalletListResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletList");
}

impl SlowWalletListResource {
    ///
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: SlowWalletListResource::module_identifier(),
            name: SlowWalletListResource::struct_identifier(),
            type_params: vec![],
        }
    }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
