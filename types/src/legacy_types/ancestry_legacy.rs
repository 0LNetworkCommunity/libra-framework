//! AutoPay view for web monitor

use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

use move_core_types::account_address::AccountAddress;

/// Struct that represents an AutoPay resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyAncestryResource {
    /// List of account addresses representing the ancestry tree
    pub tree: Vec<AccountAddress>,
}

impl MoveStructType for LegacyAncestryResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ancestry");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Ancestry");
}
impl MoveResource for LegacyAncestryResource {}
