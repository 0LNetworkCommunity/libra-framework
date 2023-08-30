use crate::move_resource::gas_coin::GAS_COIN_TYPE;
use diem_types::state_store::state_key::StateKey;
use diem_types::state_store::table::TableHandle;

use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
use std::string::FromUtf8Error;

/// Rust representation of Aggregator Move struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct Aggregator {
    handle: AccountAddress,
    key: AccountAddress,
    limit: u128,
}

impl Aggregator {
    pub fn new(handle: AccountAddress, key: AccountAddress, limit: u128) -> Self {
        Self { handle, key, limit }
    }

    /// Helper function to return the state key where the actual value is stored.
    pub fn state_key(&self) -> StateKey {
        let key_bytes = self.key.to_vec();
        StateKey::table_item(TableHandle(self.handle), key_bytes)
    }
}

/// Rust representation of Integer Move struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct Integer {
    pub value: u128,
    limit: u128,
}

/// Rust representation of OptionalAggregator Move struct.
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionalAggregator {
    pub aggregator: Option<Aggregator>,
    pub integer: Option<Integer>,
}

/// Rust representation of CoinInfo Move resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct GasCoinInfoResource {
    name: Vec<u8>,
    symbol: Vec<u8>,
    decimals: u8,
    supply: Option<OptionalAggregator>,
}

impl MoveStructType for GasCoinInfoResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("coin");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinInfo");

    fn type_params() -> Vec<TypeTag> {
        vec![GAS_COIN_TYPE.clone()]
    }
}

impl MoveResource for GasCoinInfoResource {}

impl GasCoinInfoResource {
    pub fn symbol(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.symbol.clone())
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn supply(&self) -> &Option<OptionalAggregator> {
        &self.supply
    }
}
