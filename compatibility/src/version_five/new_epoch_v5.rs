use crate::version_five::move_resource_v5::MoveStructTypeV5;
use anyhow::Result;
use move_core_types::{ident_str, identifier::IdentStr};
use serde::{Deserialize, Serialize};

/// Struct that represents a NewEpochEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewEpochEventV5 {
    epoch: u64,
}

impl NewEpochEventV5 {
    pub fn new(epoch: u64) -> Self {
        NewEpochEventV5 { epoch }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveStructTypeV5 for NewEpochEventV5 {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemConfig");
    const STRUCT_NAME: &'static IdentStr = ident_str!("NewEpochEvent");
}
