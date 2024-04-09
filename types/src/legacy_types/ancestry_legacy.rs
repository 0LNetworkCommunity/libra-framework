//! autopay view for web monitor
//! autopay view for web monitor

use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
// use diem_types::access_path::AccessPath;
// use move_core_types::language_storage::StructTag;
// use move_core_types::account_address::AccountAddress;
// use move_core_types::language_storage::CORE_CODE_ADDRESS;

use move_core_types::account_address::AccountAddress;

/// Struct that represents a AutoPay resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyAncestryResource {
    ///
    pub tree: Vec<AccountAddress>,
}

impl MoveStructType for LegacyAncestryResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ancestry");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Ancestry");
}
impl MoveResource for LegacyAncestryResource {}
