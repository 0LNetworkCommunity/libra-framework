use crate::version_five::{language_storage_v5::StructTagV5, move_resource_v5::MoveStructTypeV5};
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

use super::{language_storage_v5::CORE_CODE_ADDRESS, move_resource_v5::MoveResourceV5};
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
    pub epochs_validating_and_mining: u64,
    ///
    pub contiguous_epochs_validating_and_mining: u64,
    ///
    pub epochs_since_last_account_creation: u64,
}

impl MoveStructTypeV5 for TowerStateResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("TowerState");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TowerProofHistory");
}
impl MoveResourceV5 for TowerStateResource {}

impl TowerStateResource {
    ///
    pub fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: CORE_CODE_ADDRESS,
            module: TowerStateResource::module_identifier(),
            name: TowerStateResource::struct_identifier(),
            type_params: vec![],
        }
    }

    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
