//! tower state view for cli

use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

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

impl MoveStructType for TowerStateResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("TowerState");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TowerProofHistory");
}

impl TowerStateResource {
    ///
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
