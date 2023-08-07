//! Environment variable for 0L mode.
use once_cell::sync::Lazy;
use std::{env, str::FromStr};
use zapatos_types::chain_id::NamedChain;
/// for getting chain config from environment variables
/// in 0L abscissa apss this will override the 0L.toml file
/// in vm-genesis this will set the chain configs
pub const ENV_VAR_MODE_0L: &str = "MODE_0L";

/// The environment variable `MODE_0L` can be set to any of the the NamedChain variants MAINNET, TESTNET, LOCAL
pub static MODE_0L: Lazy<NamedChain> = Lazy::new(|| {
    let st = env::var(ENV_VAR_MODE_0L).unwrap_or_else(|_| "MAINNET".to_string());
    NamedChain::from_str(st.to_uppercase().as_str()).unwrap_or(NamedChain::MAINNET)
});
