use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};
use move_core_types::account_address::AccountAddress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryResource {
    pub list: Vec<AccountAddress>,
    pub liquidation_queue: Vec<AccountAddress>,
}

impl MoveStructType for RegistryResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("donor_voice");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Registry");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for RegistryResource {}
