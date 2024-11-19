use crate::move_resource::type_info::TypeInfo;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveStructType},
};
use serde::{Deserialize, Serialize};

/// Struct that represents a CoinRegisterEvent.
// NOTE: this is the event that indicates that an account has been created
#[derive(Debug, Serialize, Deserialize)]
pub struct CoinRegisterEvent {
    type_info: TypeInfo,
}

impl CoinRegisterEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructType for CoinRegisterEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("account");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinRegisterEvent");
}
