use zapatos_api_types::U64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GasCoin {
    pub value: U64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub coin: GasCoin,
}

impl Balance {
    pub fn new(value: u64) -> Self {
      Balance {
        coin: GasCoin {
          value: U64::from(value),
        }
      }
    }
    pub fn get(&self) -> u64 {
        *self.coin.value.inner()
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
