use crate::version_five::{
    language_storage_v5::StructTagV5, move_resource_v5::MoveResourceV5,
    move_resource_v5::MoveStructTypeV5,
};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

use super::{language_storage_v5::CORE_CODE_ADDRESS, legacy_address_v5::LegacyAddressV5};

/// Struct that represents a AutoPay resource
#[derive(Debug, Serialize, Deserialize)]
pub struct AncestryResource {
    pub tree: Vec<LegacyAddressV5>,
}

impl MoveStructTypeV5 for AncestryResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("Ancestry");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Ancestry");
}
impl MoveResourceV5 for AncestryResource {}

impl AncestryResource {
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: AncestryResource::module_identifier(),
            name: AncestryResource::struct_identifier(),
            type_params: vec![],
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
