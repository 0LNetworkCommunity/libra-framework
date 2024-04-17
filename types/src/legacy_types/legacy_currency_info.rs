// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
use anyhow::Result;
use move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};
/// Struct that represents a CurrencyInfo resource
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyInfoResource {
    total_value: u128,
    preburn_value: u64,
    to_xdx_exchange_rate: u64,
    is_synthetic: bool,
    scaling_factor: u64,
    fractional_part: u64,
    currency_code: Identifier,
    can_mint: bool,
}


impl CurrencyInfoResource {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
