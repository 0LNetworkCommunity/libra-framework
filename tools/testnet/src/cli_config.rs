use crate::cli_output::TestInfo;
use crate::config_virgin;
use anyhow::Result;
use diem_genesis::config::HostAndPort;
use diem_types::chain_id::NamedChain;
use libra_types::global_config_dir;
use std::path::PathBuf;

use crate::config_twin;

#[derive(clap::Parser)]
pub struct TestnetConfigOpts {
    /// ordered list of dns/ip with port for alice..dave, use :6180 for production validator port
    #[clap(long)]
    host: Vec<HostAndPort>,
    #[clap(short, long)]
    /// optional, location for the default data directory, otherwise will be temp
    test_dir: Option<PathBuf>,
    /// optional, of the type of chain we are starting defaults to id=2 TESTNET
    #[clap(short, long)]
    chain_name: Option<NamedChain>,
    /// path to file for legacy migration file
    #[clap(short, long)]
    migrate_legacy: Option<PathBuf>,
}

impl TestnetConfigOpts {
    pub async fn run(
        &self,
        framework_mrb_path: Option<PathBuf>,
        twin_db: Option<PathBuf>,
    ) -> Result<Vec<TestInfo>> {
        let chain_name = self.chain_name.unwrap_or(NamedChain::TESTNET); // chain_id = 2

        let data_path = self.test_dir.clone().unwrap_or_else(global_config_dir);

        let cli_out = config_virgin::setup(
            &self.host,
            chain_name,
            data_path.clone(),
            self.migrate_legacy.to_owned(),
            framework_mrb_path,
        )
        .await?;

        // if it's a twin case, then we need to do brain surgery
        if let Some(p) = twin_db {
            println!("configuring twin...");
            config_twin::configure_twin(&data_path, &p).await?;
        }

        Ok(cli_out)
    }
}
