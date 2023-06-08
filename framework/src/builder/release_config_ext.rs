use std::path::Path;
use zapatos_release_builder::ReleaseConfig;

trait LibraReleaseConfig {
  fn libra_generate_release_proposal_scripts(&self, base_path: &Path) -> anyhow::Result<()>;
}

impl LibraReleaseConfig for ReleaseConfig {
  fn libra_generate_release_proposal_scripts(&self, base_path: &Path) -> anyhow::Result<()> {
    todo!();
  }
}