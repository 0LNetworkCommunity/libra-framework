#![forbid(unsafe_code)]

use libra_framework::release::ReleaseTarget;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "libra-framework", author, version, propagate_version = true)]
enum Commands {
    /// Creates a .mrb move framework release
    Release(StandardRelease),
}

fn main() {
    let cmd: Commands = Commands::parse();
    let result = match cmd {
        Commands::Release(release) => release.execute(),
        // Commands::Custom(custom) => custom.execute(),
    };
    if let Err(e) = result {
        eprintln!("error: {:#}", e);
        std::process::exit(1)
    }
}
#[derive(Debug, Parser)]
struct  StandardRelease {
    /// The release target. One of head, devnet, testnet, or mainnet. Notice the type
    /// of target determines what packages are included in the release. For example,
    /// some packages may be available in testnet, but aren't in mainnet.
    #[clap(long, default_value = "head")]
    target: ReleaseTarget,

    /// Remove the source code from the release package to shrink its size.
    #[clap(short, long)]
    without_source_code: bool,
}

impl StandardRelease {
    fn execute(self) -> anyhow::Result<()> {
        self.target.create_release(!self.without_source_code, None)
    }
}
