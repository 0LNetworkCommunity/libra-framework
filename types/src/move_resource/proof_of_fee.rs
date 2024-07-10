use diem_sdk::move_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    move_resource::{MoveResource, MoveStructType},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsensusRewardResource {
    /// The nominal reward amount.
    pub nominal_reward: u64,
    /// The net reward amount after deductions.
    pub net_reward: u64,
    /// The entry fee required.
    pub entry_fee: u64,
    /// The clearing bid amount.
    pub clearing_bid: u64,
    /// The median winning bid amount.
    pub median_win_bid: u64,
    /// The history of median bid values.
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
