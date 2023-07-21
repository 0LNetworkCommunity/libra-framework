use move_core_types::{language_storage::StructTag, move_resource::{MoveResource, MoveStructType}, ident_str};
// use move_core_types::language_storage::StructTag;
// use zapatos_api_types::U64;
use serde::{Deserialize, Serialize};
use move_core_types::language_storage::TypeTag;
use move_core_types::identifier::IdentStr;

use once_cell::sync::Lazy;
use zapatos_types::{event::EventHandle, account_address::AccountAddress};

pub static GAS_COIN_TYPE: Lazy<TypeTag> = Lazy::new(|| {
    TypeTag::Struct(Box::new(StructTag {
        address: AccountAddress::ONE,
        module: ident_str!("gas_coin").to_owned(),
        name: ident_str!("GasCoin").to_owned(),
        type_params: vec![],
    }))
});

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
// #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct GasCoinStoreResource {
    coin: u64,
    frozen: bool,
    deposit_events: EventHandle,
    withdraw_events: EventHandle,
}

impl GasCoinStoreResource {
    pub fn new(
        coin: u64,
        frozen: bool,
        deposit_events: EventHandle,
        withdraw_events: EventHandle,
    ) -> Self {
        Self {
            coin,
            frozen,
            deposit_events,
            withdraw_events,
        }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }

    pub fn deposit_events(&self) -> &EventHandle {
        &self.deposit_events
    }

    pub fn withdraw_events(&self) -> &EventHandle {
        &self.withdraw_events
    }
}

impl MoveStructType for GasCoinStoreResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("coin");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStore");

    fn type_params() -> Vec<TypeTag> {
        vec![GAS_COIN_TYPE.clone()]
    }
}

impl MoveResource for GasCoinStoreResource {}



// TODO: This might break reading from API maybe it must be zapatos_api_types::U64;

#[derive(Debug, Serialize, Deserialize)]
pub struct GasCoin {
    pub value: u64,
}

// impl GasCoin {
//   pub fn struct_tag() -> StructTag {
//     let type_tag: TypeTag = StructTag {
//         address: CORE_CODE_ADDRESS,
//         module: Identifier::new("gas_coin").unwrap(),
//         name: Identifier::new("GasCoin").unwrap(),
//         type_params: vec![],
//     }.into();

//     StructTag {
//         address: CORE_CODE_ADDRESS,
//         module: Identifier::new("coin").unwrap(),
//         name: Identifier::new("Coin").unwrap(),
//         type_params: vec![type_tag],
//     }
//   }

// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Balance {
//     pub coin: GasCoin,
// }

// impl Balance {
//     pub fn new(value: u64) -> Self {
//       Balance {
//         coin: GasCoin {
//           value,
//         }
//       }
//     }
//     pub fn get(&self) -> u64 {
//         self.coin.value
//     }
// }

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
}
