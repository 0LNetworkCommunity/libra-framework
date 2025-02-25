use libra_framework::release::ReleaseTarget;
use std::{env::current_dir, path::PathBuf};

fn main() {
    // Set the below variable to skip the building step. This might be useful if the build
    // is broken so it can be debugged with the old outdated artifacts.
    if std::env::var("LIBRA_BUILD_FRAMEWORK").is_ok() {
        let current_dir = current_dir().expect("Should be able to get current dir");
        // Get the previous directory
        let mut prev_dir = current_dir;
        prev_dir.pop();
        println!(
            "cargo:rerun-if-changed={}",
            prev_dir.join("libra-framework").join("Move.toml").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            prev_dir.join("libra-framework").join("sources").display()
        );

        // TODO: make this run the libra binary if it is found in users $PATH
        ReleaseTarget::Head
            .create_release(
                false,
                Some(
                    PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR defined"))
                        .join("head.mrb"),
                ),
            )
            .expect("release build failed");
    }
}
