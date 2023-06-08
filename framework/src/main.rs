#![forbid(unsafe_code)]

use libra_framework::{release::ReleaseTarget, builder::{release_config_ext::libra_release_cfg_default, main_generate_proposals, framework_release_bundle}};
use clap::Parser;
use zapatos_types::account_address::AccountAddress;
use std::path::PathBuf;
use libra_framework::builder::release_config_ext;

#[derive(Parser)]
#[clap(name = "libra-framework", author, version, propagate_version = true)]
enum Commands {
    /// Creates a .mrb move framework release
    Release(GenesisRelease),
    /// Creates all artifacts for a network governance upgrade
    Upgrade(UpgradeRelease),
}

fn main() {
    let cmd: Commands = Commands::parse();
    let result = match cmd {
        Commands::Release(release) => release.execute(),
        Commands::Upgrade(release) => release.execute(),
    };
    if let Err(e) = result {
        eprintln!("error: {:#}", e);
        std::process::exit(1)
    }
}
#[derive(Debug, Parser)]
struct GenesisRelease {
    /// The release target. One of head, devnet, testnet, or mainnet. Notice the type
    /// of target determines what packages are included in the release. For example,
    /// some packages may be available in testnet, but aren't in mainnet.
    #[clap(long, default_value = "head")]
    target: ReleaseTarget,

    /// Remove the source code from the release package to shrink its size.
    #[clap(short, long)]
    without_source_code: bool,
}

impl GenesisRelease {
    fn execute(self) -> anyhow::Result<()> {
        self.target.create_release(!self.without_source_code, None)
    }
}

/// Creates all artifacts for a network governance upgrade
/// NOTE: this is an 0L reconstruction of vendor apis.
#[derive(Debug, Parser)]
struct UpgradeRelease {

    /// dir to save all the artifacts for the release.
    #[clap(short, long)]
    output_dir: PathBuf,
    /// directory of the framework source code
    /// usually ./framework
    #[clap(short, long)]
    framework_local_dir: PathBuf,

    /// TODO: optionally pass a config file with the release config
    /// if there are parameter or raw script changes.
    #[clap(short, long)]
    release_config: Option<PathBuf>,

    /// Optionally provide the framework.mrb instead of recomputing it.
    #[clap(short, long)]
    mrb_path: Option<PathBuf>,
}

impl UpgradeRelease {
    fn execute(self) -> anyhow::Result<()> {
      // we are usually only interested in upgrading the framework

      if let Some(path) = self.mrb_path {
        let bundle = framework_release_bundle::ReleaseBundle::read(path)?;
        // let release_package = bundle.release_package;
        bundle.generate_script_proposal_impl(AccountAddress::ONE, self.output_dir.clone(), None)?;
        let hash = release_config_ext::generate_hash(self.output_dir.clone(), self.framework_local_dir)?;
        Ok(())
      } else {
        let release_cfg = libra_release_cfg_default();
        main_generate_proposals::run(release_cfg, self.output_dir, self.framework_local_dir)
      }

    }
}