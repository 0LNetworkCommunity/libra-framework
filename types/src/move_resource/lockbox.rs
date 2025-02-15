//! community wallet resource
use crate::move_resource::libra_coin::LibraCoin;
use diem_types::event::EventHandle;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// Struct that represents a CommunityWallet resource
/// TODO: this is incorrect field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowWalletV2Resource {
    pub list: Vec<LockboxResource>,
    // TODO: move to global?
    pub unlock_events: EventHandle,
}
impl MoveStructType for SlowWalletV2Resource {
    const MODULE_NAME: &'static IdentStr = ident_str!("lockbox");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWalletV2");
}

impl MoveResource for SlowWalletV2Resource {}

/// Struct that represents a SlowWallet resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockboxResource {
    pub locked_coins: LibraCoin,
    pub duration_type: u64,
    pub lifetime_deposited: u64,
    pub lifetime_unlocked: u64,
    pub last_unlock_timestamp: u64,
}

impl MoveStructType for LockboxResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("lockbox");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Lockbox");
}
impl MoveResource for LockboxResource {}
