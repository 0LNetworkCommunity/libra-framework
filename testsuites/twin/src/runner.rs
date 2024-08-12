use clap::{self, Parser};
use std::{fs, path::PathBuf};

/// Twin of the network
#[derive(Parser)]

/// Set up a twin of the network, with a synced db
pub struct Twin {
    /// path of snapshot db we want marlon to drive
    #[clap(long, short)]
    pub db_dir: PathBuf,
    /// The operator.yaml file which contains registration information
    #[clap(long, short)]
    pub oper_file: Option<PathBuf>,
    /// provide info about the DB state, e.g. version
    #[clap(long, short)]
    pub info: bool,

    /// number of local validators to start
    #[clap(long, short)]
    pub count_vals: Option<u8>,
}
impl Twin {
    /// Runner for the twin
    pub async fn run(&self) -> anyhow::Result<(), anyhow::Error> {
        let db_path = fs::canonicalize(&self.db_dir)?;

        let num_validators = self.count_vals.unwrap_or(1);

        Twin::apply_with_rando_e2e(db_path, num_validators).await?;

        println!("SUCCESS: twin swarm is running. Press ctrl+c to exit.");
        std::thread::park();

        Ok(())
    }
}
