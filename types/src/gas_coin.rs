use move_core_types::{language_storage::{StructTag, CORE_CODE_ADDRESS}, identifier::Identifier};
// use move_core_types::language_storage::StructTag;
// use zapatos_api_types::U64;
use serde::{Deserialize, Serialize};
use move_core_types::language_storage::TypeTag;
#[derive(Debug, Serialize, Deserialize)]
pub struct GasCoin {
    pub value: u64, // TODO: This might break reading from API maybe it must be zapatos_api_types::U64;
}

impl GasCoin {
  pub fn struct_tag() -> StructTag {
    let type_tag: TypeTag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("gas_coin").unwrap(),
        name: Identifier::new("GasCoin").unwrap(),
        type_params: vec![],
    }.into();

    StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("coin").unwrap(),
        name: Identifier::new("Coin").unwrap(),
        type_params: vec![type_tag],
    }
  }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub coin: GasCoin,
}

impl Balance {
    pub fn new(value: u64) -> Self {
      Balance {
        coin: GasCoin {
          value,
        }
      }
    }
    pub fn get(&self) -> u64 {
        self.coin.value
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlowWalletBalance {
  pub unlocked: u64,
  pub total: u64,
}

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
