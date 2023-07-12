//! The Makewhole on-chain resource
//!
use crate::gas_coin::GasCoin;
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::MoveStructType,

};
use serde::{Deserialize, Serialize};


/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct MakeWholeResource {
    ///
    pub credits: Vec<CreditResource>,
}

impl MakeWholeResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructType for MakeWholeResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("MakeWhole");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Balance");
}


#[derive(Debug, Serialize, Deserialize)]
/// the makewhole credit resource
pub struct CreditResource {
    ///
    pub incident_name: Vec<u8>,
    ///
    pub claimed: bool,
    ///
    pub coins: GasCoin,
}

// impl MoveStructType for CreditResource {
//     const MODULE_NAME: &'static IdentStr = ident_str!("make_whole");
//     const STRUCT_NAME: &'static IdentStr = ident_str!("Credit");
// }




