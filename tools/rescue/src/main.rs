//! entry point

use clap::Parser;

fn main() -> anyhow::Result<()> {
    libra_rescue::rescue_cli::RescueCli::parse().run()
}
