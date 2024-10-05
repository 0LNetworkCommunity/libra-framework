// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::version_five::{legacy_address_v5::LegacyAddressV5, safe_serialize_v5};
use move_core_types::identifier::{IdentStr, Identifier};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const CODE_TAG: u8 = 0;
pub const RESOURCE_TAG: u8 = 1;

pub const CORE_CODE_ADDRESS: LegacyAddressV5 = LegacyAddressV5::new([
    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8,
]);

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum TypeTagV5 {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(
        #[serde(
            serialize_with = "safe_serialize_v5::type_tag_recursive_serialize",
            deserialize_with = "safe_serialize_v5::type_tag_recursive_deserialize"
        )]
        Box<TypeTagV5>,
    ),
    Struct(
        #[serde(
            serialize_with = "safe_serialize_v5::type_tag_recursive_serialize",
            deserialize_with = "safe_serialize_v5::type_tag_recursive_deserialize"
        )]
        Box<StructTagV5>,
    ),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTagV5 {
    pub address: LegacyAddressV5,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<TypeTagV5>,
}

impl StructTagV5 {
    pub fn access_vector(&self) -> Vec<u8> {
        let mut key = vec![RESOURCE_TAG];
        key.append(&mut bcs::to_bytes(self).unwrap());
        key
    }

    pub fn module_id(&self) -> ModuleId {
        ModuleId::new(self.address, self.module.to_owned())
    }
}

/// Represents the initial key into global storage where we first index by the address, and then
/// the struct tag
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct ResourceKey {
    pub address: LegacyAddressV5,
    pub type_: StructTagV5,
}

impl ResourceKey {
    pub fn address(&self) -> LegacyAddressV5 {
        self.address
    }

    pub fn type_(&self) -> &StructTagV5 {
        &self.type_
    }
}

impl ResourceKey {
    pub fn new(address: LegacyAddressV5, type_: StructTagV5) -> Self {
        ResourceKey { address, type_ }
    }
}

/// Represents the initial key into global storage where we first index by the address, and then
/// the struct tag
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct ModuleId {
    address: LegacyAddressV5,
    name: Identifier,
}

impl From<ModuleId> for (LegacyAddressV5, Identifier) {
    fn from(module_id: ModuleId) -> Self {
        (module_id.address, module_id.name)
    }
}

impl ModuleId {
    pub fn new(address: LegacyAddressV5, name: Identifier) -> Self {
        ModuleId { address, name }
    }

    pub fn name(&self) -> &IdentStr {
        &self.name
    }

    pub fn address(&self) -> &LegacyAddressV5 {
        &self.address
    }

    pub fn access_vector(&self) -> Vec<u8> {
        let mut key = vec![CODE_TAG];
        key.append(&mut bcs::to_bytes(self).unwrap());
        key
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.address, self.name)
    }
}

impl Display for StructTagV5 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "0x{}::{}::{}",
            self.address.short_str_lossless(),
            self.module,
            self.name
        )?;
        if let Some(first_ty) = self.type_params.first() {
            write!(f, "<")?;
            write!(f, "{}", first_ty)?;
            for ty in self.type_params.iter().skip(1) {
                write!(f, ", {}", ty)?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl Display for TypeTagV5 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TypeTagV5::Struct(s) => write!(f, "{}", s),
            TypeTagV5::Vector(ty) => write!(f, "vector<{}>", ty),
            TypeTagV5::U8 => write!(f, "u8"),
            TypeTagV5::U64 => write!(f, "u64"),
            TypeTagV5::U128 => write!(f, "u128"),
            TypeTagV5::Address => write!(f, "address"),
            TypeTagV5::Signer => write!(f, "signer"),
            TypeTagV5::Bool => write!(f, "bool"),
        }
    }
}

impl Display for ResourceKey {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "0x{}/{}", self.address.short_str_lossless(), self.type_)
    }
}

impl From<StructTagV5> for TypeTagV5 {
    fn from(t: StructTagV5) -> TypeTagV5 {
        TypeTagV5::Struct(Box::new(t))
    }
}
