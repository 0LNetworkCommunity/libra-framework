// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
use anyhow::Result;
use move_core_types::{
    identifier::Identifier,
};
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
    // #[serde(skip)]
    // mint_events: LegacyEventHandle,
    // #[serde(skip)]
    // burn_events: LegacyEventHandle,
    // #[serde(skip)]
    // preburn_events: LegacyEventHandle,
    // #[serde(skip)]
    // cancel_burn_events: LegacyEventHandle,
    // #[serde(skip)]
    // exchange_rate_update_events: LegacyEventHandle,
}

// #[derive(Clone, Default, Debug)]
// struct LegacyEventHandle {
//   count: u64,
//   key: Vec<u8>
// }
// impl MoveStructType for CurrencyInfoResource {
//     const MODULE_NAME: &'static IdentStr = DIEM_MODULE_IDENTIFIER;
//     const STRUCT_NAME: &'static IdentStr = ident_str!("CurrencyInfo");
// }

impl CurrencyInfoResource {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
