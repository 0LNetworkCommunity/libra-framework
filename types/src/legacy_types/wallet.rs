//! community wallet resource
use crate::legacy_types::legacy_address::LegacyAddress;
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

// NOTE: these are legacy structs for v5

/// Struct that represents a CommunityWallet resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityWalletsResourceLegacy {
    /// List
    pub list: Vec<LegacyAddress>,
}

impl MoveStructType for CommunityWalletsResourceLegacy {
    const MODULE_NAME: &'static IdentStr = ident_str!("Wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CommunityWallets");
}

impl CommunityWalletsResourceLegacy {
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

impl MoveStructType for SlowWalletResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWallet");
}

impl SlowWalletResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletListResource {
    ///
    pub list: Vec<LegacyAddress>,
}

impl MoveStructType for SlowWalletListResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletList");
}

impl SlowWalletListResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
