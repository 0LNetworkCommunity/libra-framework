use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyPledgesResource {
    pub list: Vec<PledgeAccountResource>,
}

impl MoveStructType for MyPledgesResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("pledge_accounts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("MyPledges");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for MyPledgesResource {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PledgeAccountResource {
    pub address_of_beneficiary: AccountAddress,
    pub amount: u64,
    pub pledge: u64, // use u64 instead of Coin<LibraCoin>,
    pub epoch_of_last_deposit: u64,
    pub lifetime_pledged: u64,
    pub lifetime_withdrawn: u64,
}

impl MoveStructType for PledgeAccountResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("pledge_accounts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PledgeAccount");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for PledgeAccountResource {}
