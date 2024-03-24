//! fullnode counter for system address

use anyhow::Result;

use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};
use move_core_types::move_resource::MoveResource;
use move_core_types::account_address::AccountAddress;

// Legacy Balance resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyBalanceResource {
    pub coin: u64,
}

impl LegacyBalanceResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeDepositResource {
    ///
    pub value: u64,
    ///
    pub index: u64,
    ///
    depositors: Vec<AccountAddress>,
}

impl MoveStructType for CumulativeDepositResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("cumulative_deposits");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CumulativeDeposits");
}

impl CumulativeDepositResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for CumulativeDepositResource {}
