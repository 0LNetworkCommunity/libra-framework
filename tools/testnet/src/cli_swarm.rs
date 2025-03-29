use crate::{cli_output::TestnetCliOut, twin_swarm};
use clap::{self, Parser};
use diem_framework::ReleaseBundle;
use diem_genesis::config::HostAndPort;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use std::str::FromStr;
use std::{fs, path::PathBuf};
/// Twin of the network
#[derive(Parser)]

/// Starts a quick testnet locally using Diem swarm
/// Requires that you have a `libra` binary built and
/// the envvar DIEM_FORGE_NODE_BIN_PATH pointed to it
/// e.g. DIEM_FORGE_NODE_BIN_PATH=$HOME/.cargo/bin/libra
pub struct SwarmCliOpts {
    /// number of local validators to start
    #[clap(long, short)]
    pub count_vals: Option<u8>,
}
impl SwarmCliOpts {
    /// Runner for swarm
    pub async fn run(
        &self,
        framework_mrb: ReleaseBundle,
        twin_db: Option<PathBuf>,
    ) -> anyhow::Result<TestnetCliOut> {
        let num_validators = self.count_vals.unwrap_or(2);

        let mut smoke =
            LibraSmoke::new_with_bundle(Some(num_validators), None, framework_mrb).await?;

        let out = if let Some(p) = twin_db {
            let db_path = fs::canonicalize(p)?;

            twin_swarm::awake_frankenswarm(&mut smoke, Some(db_path)).await?
        } else {
            smoke
                .swarm
                .wait_all_alive(tokio::time::Duration::from_secs(120))
                .await?;

            let api_endpoint = HostAndPort::from_str(&format!(
                "{}:{}",
                smoke.api_endpoint.host().unwrap(),
                smoke.api_endpoint.port().unwrap(),
            ))?;
            // TODO
            TestnetCliOut {
                data_dir: smoke.swarm.dir().to_path_buf(),
                api_endpoint,
                app_cfg_paths: vec![],
                private_tx_keys: vec![],
            }
        };

        println!("{}", serde_json::to_string_pretty(&out)?);

        // NOTE: all validators will stop when the LibraSmoke goes out of context. This is intentional
        // but since it's borrowed in this function you should assume it will continue until the caller goes out of scope.
        let _ = dialoguer::Input::<String>::new()
            .with_prompt(
                "\nsmoke is running `libra` processes until you exit. Press any key to exit\n{FIRE}",
            )
            .interact_text()?;

        Ok(out)
    }
}
