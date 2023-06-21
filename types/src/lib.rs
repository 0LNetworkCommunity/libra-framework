pub mod type_extensions;
pub mod util;
pub mod gas_coin;
pub mod tower;
use std::path::PathBuf;

/// default directory name for all configs
pub const LIBRA_CONFIG_DIRECTORY: &str = ".libra";

/// The default home folder
pub fn global_config_dir() -> PathBuf {
  dirs::home_dir().expect("weird...cannot get a home path").join(LIBRA_CONFIG_DIRECTORY)
} 