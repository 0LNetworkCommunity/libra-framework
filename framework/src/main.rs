#![forbid(unsafe_code)]

use clap::Parser;

use libra_framework::framework_cli::{FrameworkUpgrade, GenesisRelease, GovernanceScript};

#[derive(Parser)]
#[clap(name = "libra-framework", author, version, propagate_version = true)]
enum Commands {
    /// Creates a .mrb move framework release
    Release(GenesisRelease),
    /// Creates the script payload for a governance script
    Governance(GovernanceScript),
    /// Creates all artifacts for a network governance upgrade
    Upgrade(FrameworkUpgrade),
}

fn main() -> anyhow::Result<()> {
    let cmd: Commands = Commands::parse();
    match cmd {
        Commands::Release(release) => release.execute(),
        Commands::Upgrade(release) => release.execute(),
        Commands::Governance(release) => release.execute(),
    }
}
