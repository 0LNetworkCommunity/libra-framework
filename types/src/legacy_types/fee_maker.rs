use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeeMakerResource {
    pub epoch: u64,
    pub lifetime: u64,
}

impl MoveStructType for FeeMakerResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("fee_maker");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FeeMaker");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for FeeMakerResource {}
