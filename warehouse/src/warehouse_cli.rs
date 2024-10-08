
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

// use crate::{read_snapshot, restore, restore_bundle::RestoreBundle};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
/// DB tools e.g.: backup, restore, export to json
pub struct WarehouseCli {
    #[clap(subcommand)]
    command: Sub,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Sub {
    /// scans directory and subdirectory for manifests
    /// Tries to identify the libra version they belong to (v5 etc.)
    // #[clap(su)]
    Scan {
      #[clap(long, short('d'))]
      dir_archive: PathBuf
    },

}

impl WarehouseCli {
  pub fn run(&self) {
    match &self.command {
        Sub::Scan { dir_archive } => {
          dbg!(&dir_archive)
        },
    };
  }
}
