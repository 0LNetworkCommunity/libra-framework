use zapatos_sdk::move_types::{
  ident_str,
  identifier::IdentStr,
  language_storage::TypeTag,
  move_resource::MoveResource,
  move_resource::MoveStructType,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TowerProofHistoryView {
    ///
    pub previous_proof_hash: Vec<u8>,
    ///
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

impl MoveStructType for TowerProofHistoryView {
    const MODULE_NAME: &'static IdentStr = ident_str!("tower_state");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TowerProofHistory");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for TowerProofHistoryView {}
