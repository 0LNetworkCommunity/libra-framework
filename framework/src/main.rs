#![forbid(unsafe_code)]

use clap::Parser;

use libra_framework::framework_cli::FrameworkCli;

fn main() -> anyhow::Result<()> {
    let cmd: FrameworkCli = FrameworkCli::parse();
    match cmd {
        FrameworkCli::Release(release) => release.execute(),
        FrameworkCli::Upgrade(release) => release.execute(),
        FrameworkCli::Governance(release) => release.execute(),
    }
}
