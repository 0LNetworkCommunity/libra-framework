pub mod builder;
pub mod framework_cli;
pub mod release;
pub mod upgrade_fixtures;

//////// 0L ///////
/// Returns the release bundle for the current code.
pub fn testing_local_release_bundle() -> diem_framework::ReleaseBundle {
    release::ReleaseTarget::Head
        .load_bundle()
        .expect("could not find release bundle, head.mrb in framework/releases/head.mrb")
}

const BYTECODE_VERSION: u32 = 6;
