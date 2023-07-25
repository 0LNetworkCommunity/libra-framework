//! entry point to generate the upgrade release proposal artifacts
use crate::builder::release_config_ext::LibraReleaseConfig; // our refactored methods

use std::path::PathBuf;
use zapatos_release_builder::ReleaseConfig;


// TODO: Deprecate this method since we now start from an mrb file?
pub fn run(release_cfg: ReleaseConfig, output_dir: PathBuf, framework_local_dir: PathBuf) -> anyhow::Result<()> {
  release_cfg.libra_generate_release_proposal_scripts(output_dir.as_path(), framework_local_dir)
}
