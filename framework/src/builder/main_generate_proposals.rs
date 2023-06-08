//! entry point to generate the upgrade release proposal artifacts
use crate::builder::release_config_ext::LibraReleaseConfig; // our refactored methods

use std::path::PathBuf;
use zapatos_release_builder::ReleaseConfig;


pub fn run(release_config: PathBuf, output_dir: PathBuf, framework_local_dir: PathBuf) -> anyhow::Result<()> {
  ReleaseConfig::load_config(release_config.as_path())?
    .libra_generate_release_proposal_scripts(output_dir.as_path(), framework_local_dir)
}
