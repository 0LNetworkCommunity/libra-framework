pub mod builder;
pub mod framework_cli;
pub mod release;
pub mod upgrade_fixtures;

//////// 0L ///////
/// Returns the release bundle for the current code.
pub fn head_release_bundle() -> diem_framework::ReleaseBundle {
    release::ReleaseTarget::Head
        .load_bundle()
        .expect("release build failed")
}

/// gets a mbr file equivalent to what is currently on mainnet
pub fn mainnet_compat_release_bundle() -> diem_framework::ReleaseBundle {
    release::ReleaseTarget::Mainnet
        .load_bundle()
        .expect("release build failed")
}

const BYTECODE_VERSION: u32 = 6;
