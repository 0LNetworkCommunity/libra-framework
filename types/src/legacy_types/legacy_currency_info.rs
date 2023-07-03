// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::{
    identifier::Identifier,
};
use serde::{Deserialize, Serialize};
use zapatos_types::event::EventHandle;
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
    mint_events: EventHandle,
    burn_events: EventHandle,
    preburn_events: EventHandle,
    cancel_burn_events: EventHandle,
    exchange_rate_update_events: EventHandle,
}

// impl MoveStructType for CurrencyInfoResource {
//     const MODULE_NAME: &'static IdentStr = DIEM_MODULE_IDENTIFIER;
//     const STRUCT_NAME: &'static IdentStr = ident_str!("CurrencyInfo");
// }

impl CurrencyInfoResource {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}
