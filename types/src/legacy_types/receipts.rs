//! fullnode counter for system address

use move_core_types::account_address::AccountAddress;
use move_core_types::move_resource::MoveResource;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptsResource {
    ///
    pub destination: Vec<AccountAddress>,
    ///
    pub cumulative: Vec<u64>,
    ///
    pub last_payment_timestamp: Vec<u64>,
    ///
    pub last_payment_value: Vec<u64>,
}

impl MoveStructType for ReceiptsResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("receipts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UserReceipts");
}

impl MoveResource for ReceiptsResource {}
