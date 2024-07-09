use diem_sdk::move_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    move_resource::{MoveResource, MoveStructType},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JailResource {
    pub is_jailed: bool,
    // number of times the validator was dropped from set. Does not reset.
    pub lifetime_jailed: u64,
    // number of times a downstream validator this user has vouched for has
    // been jailed.
    // this is recursive. So if a validator I vouched for, vouched for
    // a third validator that failed, this number gets incremented.
    pub lifetime_vouchees_jailed: u64,
    // validator that was jailed and qualified to enter the set, but fails
    // to complete epoch.
    // this resets as soon as they rejoin successfully.
    // this counter is used for ordering prospective validators entering a set.
    pub consecutive_failure_to_rejoin: u64,
}

impl MoveStructType for JailResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("jail");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Jail");

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }
}

impl MoveResource for JailResource {}
