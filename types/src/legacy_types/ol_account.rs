use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurnTrackerResource {
    pub prev_supply: u64,
    pub prev_balance: u64,
    pub burn_at_last_calc: u64,
    pub cumu_burn: u64,
}

impl MoveStructType for BurnTrackerResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ol_account");
    const STRUCT_NAME: &'static IdentStr = ident_str!("BurnTracker");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for BurnTrackerResource {}
