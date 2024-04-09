use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};
use move_core_types::account_address::AccountAddress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyVouchesResource {
    ///
    pub my_buddies: Vec<AccountAddress>,
    ///
    pub epoch_vouched: Vec<u64>,
}

impl MoveStructType for MyVouchesResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("vouch");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MyVouches");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for MyVouchesResource {}
