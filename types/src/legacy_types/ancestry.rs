//! autopay view for web monitor
//! autopay view for web monitor

use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

use super::legacy_address::LegacyAddress;

/// Struct that represents a AutoPay resource
#[derive(Debug, Serialize, Deserialize)]
pub struct AncestryResource {
    ///
    pub tree: Vec<LegacyAddress>,
}

impl MoveStructType for AncestryResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("Ancestry");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Ancestry");
}
impl MoveResource for AncestryResource {}

impl AncestryResource {
    // ///
    // pub fn struct_tag() -> StructTag {
    //     StructTag {
    //         address: CORE_CODE_ADDRESS,
    //         module: AncestryResource::module_identifier(),
    //         name: AncestryResource::struct_identifier(),
    //         type_params: vec![],
    //     }
    // }
    // ///
    // pub fn access_path(account: AccountAddress) -> AccessPath {
    //     let resource_key = ResourceKey::new(account, AncestryResource::struct_tag());
    //     AccessPath::resource_access_path(resource_key)
    // }
    // ///
    // pub fn resource_path() -> Vec<u8> {
    //     AccessPath::resource_access_vec(AncestryResource::struct_tag())
    // }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

}
