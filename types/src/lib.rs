//! Exprorting some types from vendor so that they can be used in other crates
pub mod util;
pub mod gas_coin;
// pub mod tower;
pub mod type_extensions;
pub mod exports;
pub mod legacy_types;
// #[cfg(test)]
pub mod test_drop_helper;

use std::path::PathBuf;

/// default directory name for all configs
pub const LIBRA_CONFIG_DIRECTORY: &str = ".libra";

/// The default home folder
pub fn global_config_dir() -> PathBuf {
  dirs::home_dir().expect("weird...cannot get a home path").join(LIBRA_CONFIG_DIRECTORY)
} 