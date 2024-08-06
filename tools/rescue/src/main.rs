//! entry point
use libra_rescue::rescue_cli::RescueCli;

fn main() -> anyhow::Result<()> {
    RescueCli::parse().run();
}
