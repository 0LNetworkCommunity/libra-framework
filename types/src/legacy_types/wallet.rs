//! community wallet resource
use crate::legacy_types::legacy_address::LegacyAddress;
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};
use crate::legacy_types::legacy_miner_state::TowerStateResource;
use move_core_types::account_address::AccountAddress;
// NOTE: these are legacy structs for v5

/// Struct that represents a CommunityWallet resource
/// TODO: this is incorrect field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityWalletsResourceLegacy {
    // List
    pub list: Vec<LegacyAddress>,
}

impl MoveStructType for CommunityWalletsResourceLegacy {
    const MODULE_NAME: &'static IdentStr = ident_str!("community_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CommunityWallet");
}

impl CommunityWalletsResourceLegacy {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        println!("community bytes: {:?}", bytes.len());
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for CommunityWalletsResourceLegacy {}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlowWalletResource {
    ///
    pub unlocked: u64,
    ///
    pub transferred: u64,
}

impl MoveStructType for SlowWalletResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("slow_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWallet");
}

impl SlowWalletResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for SlowWalletResource {}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlowWalletListResource {
    ///
    pub list: Vec<AccountAddress>,
    // TODO: add drip events
}

impl MoveStructType for SlowWalletListResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("slow_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletList");
}

impl SlowWalletListResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for SlowWalletListResource {}