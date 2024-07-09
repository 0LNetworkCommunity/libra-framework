//! fullnode counter for system address

use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptsResource {
    /// List of destination addresses for the payments.
    pub destination: Vec<AccountAddress>,
    /// Cumulative amounts sent to each destination.
    pub cumulative: Vec<u64>,
    /// Timestamps of the last payment to each destination.
    pub last_payment_timestamp: Vec<u64>,
    /// Values of the last payment to each destination.
    pub last_payment_value: Vec<u64>,
}

impl MoveStructType for ReceiptsResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("receipts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UserReceipts");
}

impl MoveResource for ReceiptsResource {}
