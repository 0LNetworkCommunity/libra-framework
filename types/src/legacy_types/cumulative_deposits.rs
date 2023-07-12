//! fullnode counter for system address

use anyhow::Result;

use move_core_types::{
    ident_str,
    identifier::IdentStr,

    move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

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
}

impl MoveStructType for CumulativeDepositResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UserReceipts");
}

impl CumulativeDepositResource {

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
