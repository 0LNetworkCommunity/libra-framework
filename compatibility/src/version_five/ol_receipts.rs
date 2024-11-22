use crate::version_five::{language_storage_v5::StructTagV5, move_resource_v5::MoveStructTypeV5};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

use super::{language_storage_v5::CORE_CODE_ADDRESS, legacy_address_v5::LegacyAddressV5};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptsResource {
    ///
    pub destination: Vec<LegacyAddressV5>,
    ///
    pub cumulative: Vec<u64>,
    ///
    pub last_payment_timestamp: Vec<u64>,
    ///
    pub last_payment_value: Vec<u64>,
}

impl MoveStructTypeV5 for ReceiptsResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("Receipts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UserReceipts");
}

impl ReceiptsResource {
    ///
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: ReceiptsResource::module_identifier(),
            name: ReceiptsResource::struct_identifier(),
            type_params: vec![],
        }
    }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
