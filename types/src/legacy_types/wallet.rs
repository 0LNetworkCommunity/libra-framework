//! community wallet resource
use diem_types::event::EventHandle;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
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
    /// Amount unlocked.
    pub unlocked: u64,
    /// Amount transferred.
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
    /// List of slow wallet addresses.
    pub list: Vec<AccountAddress>,
    /// Event handle for drip events.
    pub drip_events: EventHandle,
}

impl MoveStructType for SlowWalletListResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("slow_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletList");
}

impl MoveResource for SlowWalletListResource {}
