//! community wallet resource
use super::legacy_address::LegacyAddress;
use move_core_types::move_resource::MoveResource;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};
// NOTE: these are legacy structs for v5

/// Struct that represents a CommunityWallet resource
/// TODO: this is incorrect field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityWalletsResourceLegacyV5 {
    // List
    pub list: Vec<LegacyAddress>,
}

impl MoveStructType for CommunityWalletsResourceLegacyV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("community_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CommunityWallet");
}

impl MoveResource for CommunityWalletsResourceLegacyV5 {}
