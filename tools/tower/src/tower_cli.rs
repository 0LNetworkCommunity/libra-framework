
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]

/// clap struct entry point for the tower cli
pub struct TowerCli {
    #[clap(subcommand)]
    subcommand: TowerSub,
}

#[derive(Subcommand)]
enum TowerSub {
  Test,
}

impl TowerCli {
    pub async fn run(&self) -> anyhow::Result<()>{
        todo!()
    }
}