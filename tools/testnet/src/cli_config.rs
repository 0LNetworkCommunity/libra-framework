use crate::cli_output::TestnetCliOut;
use crate::config_virgin;
use anyhow::Result;
use diem_genesis::config::HostAndPort;
use diem_types::chain_id::NamedChain;
use libra_types::{core_types::fixtures::TestPersona, global_config_dir};
use std::path::PathBuf;

use crate::config_twin;

#[derive(clap::Parser)]
pub struct TestnetConfigOpts {
    #[clap(short, long)]
    /// location for the default data directory
    out_dir: Option<PathBuf>,
    /// name of the type of chain we are starting
    #[clap(short, long)]
    chain_name: Option<NamedChain>,
    /// sensible defaults for testnet, does not need a genesis repo
    /// accounts are created from fixture mnemonics for alice, bob, carol, dave
    /// which persona is this machine going to register as
    #[clap(short, long)]
    me: TestPersona,
    /// ordered list of dns/ip with port for alice..dave
    /// use 6180 for production validator port
    #[clap(short('i'), long)]
    host_list: Vec<HostAndPort>,
    /// path to file for legacy migration file
    #[clap(short, long)]
    json_legacy: Option<PathBuf>,
}

impl TestnetConfigOpts {
    pub async fn run(
        &self,
        framework_mrb_path: Option<PathBuf>,
        twin_db: Option<PathBuf>,
    ) -> Result<TestnetCliOut> {
        let chain_name = self.chain_name.unwrap_or(NamedChain::TESTNET); // chain_id = 2

        // let data_path = self.out_dir.clone().unwrap_or_else(|| {
        //     let p = global_config_dir().join("swarm");
        //     println!("using default path: {}", p.display());
        //     p
        // });

        let data_path = self.out_dir.clone().unwrap_or_else(global_config_dir);

        let cli_out = config_virgin::setup(
            &self.me,
            &self.host_list,
            chain_name,
            data_path,
            self.json_legacy.to_owned(),
            framework_mrb_path,
        )
        .await?;

        // if it's a twin case, then we need to do brain surgery
        if let Some(p) = twin_db {
            println!("configuring twin...");
            config_twin::configure_twin(&cli_out.data_dir, &p).await?;
        }

        Ok(cli_out)
    }
}
