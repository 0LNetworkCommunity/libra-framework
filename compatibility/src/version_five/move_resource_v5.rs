// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0
use super::{
    language_storage_v5::{StructTagV5, TypeTagV5},
    legacy_address_v5::{LegacyAddressV5, LEGACY_CORE_CODE_ADDRESS},
};
use move_core_types::identifier::{IdentStr, Identifier};
use serde::de::DeserializeOwned;

pub trait MoveStructTypeV5 {
    const ADDRESS: LegacyAddressV5 = LEGACY_CORE_CODE_ADDRESS;
    const MODULE_NAME: &'static IdentStr;
    const STRUCT_NAME: &'static IdentStr;

    fn module_identifier() -> Identifier {
        Self::MODULE_NAME.to_owned()
    }

    fn struct_identifier() -> Identifier {
        Self::STRUCT_NAME.to_owned()
    }

    fn type_params() -> Vec<TypeTagV5> {
        vec![]
    }

    fn struct_tag() -> StructTagV5 {
        StructTagV5 {
            address: Self::ADDRESS,
            name: Self::struct_identifier(),
            module: Self::module_identifier(),
            type_params: Self::type_params(),
        }
    }
}

pub trait MoveResourceV5: MoveStructTypeV5 + DeserializeOwned {
    fn resource_path() -> Vec<u8> {
        Self::struct_tag().access_vector()
    }
}
