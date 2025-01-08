use clap::{self, Parser};
use libra_smoke_tests::libra_smoke::LibraSmoke;
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

        let mut smoke = LibraSmoke::new(Some(num_validators), None).await?;
        // save_cli_config_all(&mut smoke.swarm)?;

        // thread::sleep(Duration::from_secs(60));
        Twin::make_twin_swarm(&mut smoke, Some(db_path), true).await?;

        Ok(())
    }
}
