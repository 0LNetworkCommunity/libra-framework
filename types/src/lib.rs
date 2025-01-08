//! Exprorting some types from vendor so that they can be used in other crates
pub mod core_types;
pub mod exports;
pub mod move_resource;
pub mod ol_progress;
pub mod type_extensions;
pub mod util;

use std::path::PathBuf;

/// default directory name for all configs
pub const GLOBAL_CONFIG_DIRECTORY_0L: &str = ".libra";

/// The default home folder
pub fn global_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("weird...cannot get a home path")
        .join(GLOBAL_CONFIG_DIRECTORY_0L)
}

/// The coin scaling or decimal represenation of 1 coin in the VM. 1 coin is divisible by 1,000,000, or 6 decimals precision.
pub const ONCHAIN_DECIMAL_PRECISION: u8 = 6;
