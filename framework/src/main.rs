#![forbid(unsafe_code)]

use libra_framework::release::ReleaseTarget;
use clap::Parser;
use std::path::PathBuf;

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
    /// The file path the the configuration file for the release.
    #[clap(short, long)]
    release_config: PathBuf,
    /// dir to save all the artifacts for the release.
    #[clap(short, long)]
    output_dir: PathBuf,
    /// directory of the framework source code
    /// usually ./framework
    #[clap(short, long)]
    framework_local_dir: PathBuf
}

impl UpgradeRelease {
    fn execute(self) -> anyhow::Result<()> {
      todo!()
        // self.target.create_release(!self.without_source_code, None)
    }
}