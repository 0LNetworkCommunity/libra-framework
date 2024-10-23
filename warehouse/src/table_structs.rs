use libra_types::exports::AccountAddress;

#[derive(Debug, Clone)]
/// The basic information for an account
pub struct WarehouseState {
  pub account: WarehouseAccount,
  pub balance: Option<WarehouseBalance>,
}

#[derive(Debug, Clone)]
pub struct WarehouseAccount {
  pub address: AccountAddress,
}

#[derive(Debug, Clone)]
pub struct WarehouseBalance {
  // balances in v6+ terms
  pub balance: u64,
  // the balance pre v6 recast
  pub legacy_balance: Option<u64>,

}
