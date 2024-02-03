//! upgrade release builder module
//! This is a refactoring of the diem-move/diem-release-builder/src/main.rs
//! Framework upgrades in vendor code has many hard coded values, and it is not easy to extend. Plus many of the cli tools have tight coupling to the framework
//! source code in the monorepo. sigh.
//! So this module tries to dissassemble cli, and rebuild with some custom
//! traits so that we can pass the framework source from an arbitrary path.
//! There is also code generation which is hard coded with strings (double sigh) in release_bundle, which needs to be renamed if the framework is renamed.
// pub mod main_generate_proposals; // this is the entry point for the cli. see diem-move/diem-release-builder/src/main.rs
// pub mod release_entry_ext; // a trait to extend the release entry struct see diem-move/diem-release-builder/src/components/mod.rs
// pub mod release_config_ext; // a trait to extend the release config struct see diem-move/diem-release-builder/src/components/mod.rs
pub mod framework_generate_upgrade_proposal; // see diem-move/diem-release-builder/src/components/framework.rs
pub mod framework_release_bundle; // note this lives in a different module in vendor. see diem-move/framework/src/release_bundle.rs
pub mod named_addresses; // OL uses different names for addresses
pub mod release; // OL uses different names for addresses
