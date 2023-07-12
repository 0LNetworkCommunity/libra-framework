//! validator config view for web monitor

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

//// TODO THIS IS DUPLICATED WITH types/src/validator_config.rs
/// Please rename.

/// Struct that represents a Validator Config resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfigResource {
    ///
    pub config: Option<ConfigResource>,
    ///
    pub operator_account: Option<AccountAddress>,
    ///
    pub human_name: Vec<u8>,
}

/// Struct that represents a Config resource
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ConfigResource {
    ///
    pub consensus_pubkey: Vec<u8>,
    ///
    pub validator_network_addresses: Vec<u8>,
    ///
    pub fullnode_network_addresses: Vec<u8>,
}

impl MoveStructType for ValidatorConfigResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ValidatorConfig");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ValidatorConfig");
}
// impl MoveResource for ValidatorConfigResource {}

impl ValidatorConfigResource {


    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }

    // ///
    // pub fn get_view(&self) -> ValidatorConfigView {
    //     ValidatorConfigView {
    //         operator_account: self.operator_account.clone(),
    //         operator_has_balance: None,
    //     }
    // }
}

/// Struct that represents a view for Validator Config view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfigView {
    ///
    pub operator_account: Option<AccountAddress>,
    ///
    pub operator_has_balance: Option<bool>,
}
