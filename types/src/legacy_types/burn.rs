use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBurnPreferenceResource {
    ///
    pub send_community: bool,
}


impl MoveStructType for UserBurnPreferenceResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("burn");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UserBurnPreference");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for UserBurnPreferenceResource {}
