use diem_sdk::move_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    move_resource::{MoveResource, MoveStructType},
};
use move_core_types::account_address::AccountAddress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidatorUniverseResource {
    validators: Vec<AccountAddress>,
}

impl MoveStructType for ValidatorUniverseResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("validator_universe");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ValidatorUniverse");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for ValidatorUniverseResource {}
