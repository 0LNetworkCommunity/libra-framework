//! tower state view for cli

use move_core_types::move_resource::MoveResource;
use move_core_types::{ident_str, identifier::IdentStr, move_resource::MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TowerStateResourceV5 {
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
}

impl MoveStructType for TowerStateResourceV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("tower_state");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TowerProofHistory");
}

impl MoveResource for TowerStateResourceV5 {}
