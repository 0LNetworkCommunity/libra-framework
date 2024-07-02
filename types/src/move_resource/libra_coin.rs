use crate::move_resource::gas_coin::{cast_coin_to_decimal, GAS_COIN_TYPE};
use diem_types::event::EventHandle;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::TypeTag,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize, Clone)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct LibraCoinStoreResource {
    coin: u64,
    deposit_events: EventHandle,
    withdraw_events: EventHandle,
}

impl LibraCoinStoreResource {
    pub fn new(coin: u64, deposit_events: EventHandle, withdraw_events: EventHandle) -> Self {
        Self {
            coin,
            deposit_events,
            withdraw_events,
        }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }

    pub fn deposit_events(&self) -> &EventHandle {
        &self.deposit_events
    }

    pub fn withdraw_events(&self) -> &EventHandle {
        &self.withdraw_events
    }
}

impl MoveStructType for LibraCoinStoreResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("coin");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStore");

    fn type_params() -> Vec<TypeTag> {
        vec![GAS_COIN_TYPE.clone()]
    }
}

impl MoveResource for LibraCoinStoreResource {}

// TODO: This might break reading from API maybe it must be diem_api_types::U64;

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraCoin {
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletBalance {
    pub unlocked: u64,
    pub total: u64,
}

impl MoveStructType for SlowWalletBalance {
    const MODULE_NAME: &'static IdentStr = ident_str!("slow_wallet");
    const STRUCT_NAME: &'static IdentStr = ident_str!("SlowWallet");
}

impl MoveResource for SlowWalletBalance {}

impl SlowWalletBalance {
    pub fn from_value(value: Vec<serde_json::Value>) -> anyhow::Result<Self> {
        if value.len() != 2 {
            return Err(anyhow::anyhow!("invalid value length"));
        }
        let unlocked = serde_json::from_value::<String>(value[0].clone())?.parse::<u64>()?;
        let total = serde_json::from_value::<String>(value[1].clone())?.parse::<u64>()?;

        Ok(Self { unlocked, total })
    }

    // scale it to include decimals
    pub fn scaled(&self) -> LibraBalanceDisplay {
        LibraBalanceDisplay {
            unlocked: cast_coin_to_decimal(self.unlocked),
            total: cast_coin_to_decimal(self.total),
        }
    }
}

/// This is the same shape as Slow Wallet balance, except that it is scaled.
/// The slow wallet struct contains the coin value as it exists in the database which is without decimals. The decimal precision for LibraCoin is 6. So we need to scale it for human consumption.
#[derive(Debug, Serialize, Deserialize)]

pub struct LibraBalanceDisplay {
    pub unlocked: f64,
    pub total: f64,
}
