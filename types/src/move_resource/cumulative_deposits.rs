//! fullnode counter for system address

use anyhow::Result;

use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

// Legacy Balance resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyBalanceResource {
    pub coin: u64,
}

impl LegacyBalanceResource {
    /// Attempts to deserialize bytes into a `LegacyBalanceResource` instance
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeDepositResource {
    /// Total deposited value
    pub value: u64,
    /// Index of the cumulative deposits
    pub index: u64,
    /// List of depositors' account addresses
    depositors: Vec<AccountAddress>,
}

impl MoveStructType for CumulativeDepositResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("cumulative_deposits");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CumulativeDeposits");
}

impl MoveResource for CumulativeDepositResource {}
