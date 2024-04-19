use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsensusRewardResource {
    ///
    pub nominal_reward: u64,
    ///
    pub net_reward: u64,
    ///
    pub entry_fee: u64,
    ///
    pub clearing_bid: u64,
    ///
    pub median_win_bid: u64,
    ///
    pub median_history: Vec<u64>,
}

impl MoveStructType for ConsensusRewardResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("proof_of_fee");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ConsensusReward");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for ConsensusRewardResource {}
