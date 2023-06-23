//! tower state view for cli

// use crate::{
//     access_path::AccessPath,
//     account_config::constants:: CORE_CODE_ADDRESS,
// };
use anyhow::Result;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{StructTag},
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
use bcs;
use zapatos_types::{
  account_config::CORE_CODE_ADDRESS,
};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Serialize, Deserialize)]
pub struct TowerStateResource {
    ///
    pub previous_proof_hash: Vec<u8>,
    /// user's latest verified_tower_height
    pub verified_tower_height: u64, 
    ///
    pub latest_epoch_mining: u64,
    ///
    pub count_proofs_in_epoch: u64,
    ///
    pub epochs_mining: u64,
    ///
    pub contiguous_epochs_mining: u64,
    
    // pub epochs_since_last_account_creation: u64,
}

impl MoveStructType for TowerStateResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("TowerState");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TowerProofHistory");
}
impl MoveResource for TowerStateResource {}

impl TowerStateResource {
    ///
    pub fn struct_tag() -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: TowerStateResource::module_identifier(),
            name: TowerStateResource::struct_identifier(),
            type_params: vec![],
        }
    }
    ///
    // pub fn access_path(account: AccountAddress) -> AccessPath {
    //     // let resource_key = ResourceKey::new(
    //     //     account,
    //     //     TowerStateResource::struct_tag(),
    //     // );
    //     AccessPath::resource_access_path(account)
    // }
    ///
    // pub fn resource_path() -> Vec<u8> {
    //     AccessPath::resource_access_vec(TowerStateResource::struct_tag())
    // }

    /// 
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
