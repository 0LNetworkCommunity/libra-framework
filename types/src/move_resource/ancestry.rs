//! autopay view for web monitor

use anyhow::Result;
use diem_types::account_address::AccountAddress;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// Struct that represents a AutoPay resource
#[derive(Debug, Serialize, Deserialize)]
pub struct AncestryResource {
    ///
    pub tree: Vec<AccountAddress>,
}

impl MoveStructType for AncestryResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ancestry");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Ancestry");
}
impl MoveResource for AncestryResource {}

impl AncestryResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
