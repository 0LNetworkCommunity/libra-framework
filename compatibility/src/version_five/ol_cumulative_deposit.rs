
use crate::version_five::{language_storage_v5::StructTagV5, move_resource_v5::MoveStructTypeV5};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

use super::language_storage_v5::CORE_CODE_ADDRESS;

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeDepositResource {
    ///
    pub value: u64,
    ///
    pub index: u64,
}

impl MoveStructTypeV5 for CumulativeDepositResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CumulativeDeposits");
}

impl CumulativeDepositResource {
    ///
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: CumulativeDepositResource::module_identifier(),
            name: CumulativeDepositResource::struct_identifier(),
            type_params: vec![],
        }
    }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
