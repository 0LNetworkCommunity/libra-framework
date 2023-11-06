//! fullnode counter for system address

use anyhow::Result;
use super::legacy_address::LegacyAddress;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptsResource {
    ///
    pub destination: Vec<LegacyAddress>,
    ///
    pub cumulative: Vec<u64>,
    ///
    pub last_payment_timestamp: Vec<u64>,
    ///
    pub last_payment_value: Vec<u64>,
}

impl MoveStructType for ReceiptsResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("Receipts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UserReceipts");
}

impl ReceiptsResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}