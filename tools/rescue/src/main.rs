//! entry point

use clap::Parser;

fn main() -> anyhow::Result<()> {
    libra_rescue::cli_main::RescueCli::parse().run()
}
