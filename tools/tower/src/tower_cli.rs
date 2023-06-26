use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), author, version, about, long_about = None, arg_required_else_help = true)]

/// clap struct entry point for the tower cli
pub struct TowerCli {
    #[clap(subcommand)]
    command: TowerSub,
}

#[derive(Subcommand)]
enum TowerSub {
  Backlog,
  Start,
  Test,
  Zero,
}

impl TowerCli {
    pub async fn run(&self) -> anyhow::Result<()>{
      let cli = TowerCli::parse();
      match cli.command {
        TowerSub::Backlog => {
          println!("backlog");
        },
        TowerSub::Start => {
          println!("start");
        },
        TowerSub::Test => {
          println!("test");
        },
        TowerSub::Zero => {
          println!("zero");
        },
      }
      Ok(())
    }
}