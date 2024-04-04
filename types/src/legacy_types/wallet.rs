//! community wallet resource
use diem_types::event::EventHandle;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};
use move_core_types::account_address::AccountAddress;
// NOTE: these are legacy structs for v5

/// Struct that represents a CommunityWallet resource
/// TODO: this is incorrect field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityWalletsResourceLegacy {
    // List
    pub list: Vec<AccountAddress>,
}

impl MoveStructType for CommunityWalletsResourceLegacy {
    const MODULE_NAME: &'static IdentStr = ident_str!("community_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CommunityWallet");
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
impl MoveResource for SlowWalletResource {}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlowWalletListResource {
    ///
    pub list: Vec<AccountAddress>,
    ///
    pub drip_events: EventHandle
}

impl MoveStructType for SlowWalletListResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("slow_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletList");
}

impl MoveResource for SlowWalletListResource {}