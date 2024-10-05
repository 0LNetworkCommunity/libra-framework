// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    // language_storage::{StructTagV5, TypeTag},
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
use crate::version_five::language_storage_v5::{StructTagV5, TypeTagV5};
use crate::version_five::legacy_address_v5::{LegacyAddressV5, LEGACY_CORE_CODE_ADDRESS};
use crate::version_five::move_resource_v5::MoveStructTypeV5;
use crate::version_five::move_resource_v5::MoveResourceV5;
/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResourceV5 {
    coin: u64,
}

impl BalanceResourceV5 {
    pub fn new(coin: u64) -> Self {
        Self { coin }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }
}

impl MoveStructTypeV5 for BalanceResourceV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemAccount");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Balance");

    fn type_params() -> Vec<TypeTagV5> {
        vec![TypeTagV5::Struct(Box::new(StructTagV5 {
            address: LEGACY_CORE_CODE_ADDRESS,
            module: ident_str!("GAS").into(),
            name: ident_str!("GAS").into(),
            type_params: vec![],
        }))]
    }
}

impl MoveResourceV5 for BalanceResourceV5 {}

// pub fn xus_tag() -> TypeTag {
//     TypeTag::Struct(Box::new(StructTagV5 {
//         address: AccountAddress::ONE,
//         module: ident_str!("GAS").into(),
//         name: ident_str!("GAS").into(),
//         type_params: vec![],
//     }))
// }
