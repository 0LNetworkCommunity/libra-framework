use libra_types::exports::AccountAddress;

#[derive(Debug, Clone)]
/// The basic information for an account
pub struct WarehouseRecord {
  pub account: WarehouseAccount,
  pub time: WarehouseTime,
  pub balance: Option<WarehouseBalance>,
}

impl WarehouseRecord {
  pub fn new(address: AccountAddress) -> Self {
    Self {
      account: WarehouseAccount {
        address,
      },
      time: WarehouseTime::default(),
      balance: Some(WarehouseBalance::default()),
    }
  }
  pub fn set_time(&mut self, timestamp: u64, version: u64, epoch: u64) {
    self.time.timestamp = timestamp;
    self.time.version = version;
    self.time.epoch = epoch;
  }
}

// holds timestamp, chain height, and epoch
#[derive(Debug, Clone, Default)]
pub struct WarehouseTime {
  pub timestamp: u64,
  pub version: u64,
  pub epoch: u64,
}
#[derive(Debug, Clone)]
pub struct WarehouseAccount {
  pub address: AccountAddress,
}

#[derive(Debug, Default, Clone, sqlx::FromRow)]
pub struct WarehouseBalance {
  // balances in v6+ terms
  #[sqlx(try_from = "i64")]
  pub balance: u64,
  // the balance pre v6 recast
  #[sqlx(default)]
  pub legacy_balance: Option<u64>,
}
