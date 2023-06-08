
use std::path::PathBuf;

pub fn create_config(release_config: PathBuf, output_dir: PathBuf) -> anyhow::Result<()> {
  zapatos_release_builder::ReleaseConfig::load_config(release_config.as_path())?
            .generate_release_proposal_scripts(output_dir.as_path())
}

