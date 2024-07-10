//! validator config view for web monitor

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// TODO THIS IS DUPLICATED WITH types/src/validator_config.rs
/// Please rename.

/// Struct that represents a Validator Config resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfigResource {
    /// The config resource for the validator.
    pub config: Option<ConfigResource>,
    /// The account address of the operator.
    pub operator_account: Option<AccountAddress>,
    /// The human-readable name of the validator.
    pub human_name: Vec<u8>,
}

/// Struct that represents a Config resource
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ConfigResource {
    /// The consensus public key.
    pub consensus_pubkey: Vec<u8>,
    /// The network addresses for the validator.
    pub validator_network_addresses: Vec<u8>,
    /// The network addresses for the fullnode.
    pub fullnode_network_addresses: Vec<u8>,
}

impl MoveStructType for ValidatorConfigResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("stake");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ValidatorConfig");
}
impl MoveResource for ValidatorConfigResource {}

impl ValidatorConfigResource {
    /// Tries to create a `ValidatorConfigResource` from a byte array.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

/// Struct that represents a view for Validator Config view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfigView {
    /// The account address of the operator.
    pub operator_account: Option<AccountAddress>,
    /// Indicates if the operator has a balance.
    pub operator_has_balance: Option<bool>,
}
