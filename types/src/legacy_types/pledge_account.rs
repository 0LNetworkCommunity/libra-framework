use diem_sdk::move_types::{
    ident_str, identifier::IdentStr, language_storage::TypeTag, move_resource::MoveResource,
    move_resource::MoveStructType,
};
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyPledgesResource {
    list: Vec<PledgeAccountResource>,
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
    // project_id: vector<u8>, // a string that identifies the project
    address_of_beneficiary: AccountAddress,
    // the address of the project, also where the BeneficiaryPolicy is stored for reference.
    amount: u64,
    pledge: u64 , // use u64 instead of Coin<LibraCoin>,
    epoch_of_last_deposit: u64,
    lifetime_pledged: u64,
    lifetime_withdrawn: u64,
}

impl MoveStructType for PledgeAccountResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("pledge_accounts");
    const STRUCT_NAME: &'static IdentStr = ident_str!("PledgeAccount");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for PledgeAccountResource {}
