
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
/// Start a libra node
pub struct MissionOpts {
    #[clap(short, long)]
    /// directory enclosing the `/db` folder of the node
    data_path: PathBuf,
    #[clap(short, long)]
    /// directory to read/write or the rescue.blob. Will default to db_path/rescue.blob
    tx_path: Option<PathBuf>,
    #[clap(short, long)]
    /// directory to read/write or the rescue.blob
    script_path: Option<PathBuf>,
    #[clap(long)]
    /// directory to read/write or the rescue.blob
    framework_path: Option<PathBuf>,
}

impl MissionOpts {
  pub async fn run(&self) -> anyhow::Result<()> {

    // There are two options:
    // 1. upgrade the framework because the source in db is a brick.
    // 2. the framework in DB is usable, and we need to execute an admin
    //    transaction from a .move source

    if self.script_path.is_none() && self.framework_path.is_none() {
      bail!("no options provided, need a --framework-path or a --script-path");
    }

    if let Some(p) = self.script_path {
      custom_script(
        &p,

      )
    }


    Ok(())
  }
}