use crate::make_twin_swarm;
use clap::{self, Parser};
use diem_framework::ReleaseBundle;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use std::{fs, path::PathBuf};
/// Twin of the network
#[derive(Parser)]

/// Starts a quick testnet locally using Diem swarm
/// Requires that you have a `libra` binary built and
/// the ennvar DIEM_FORGE_NODE_BIN_PATH pointed to it
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
        twin_db: Option<PathBuf>,
        framework_mrb: ReleaseBundle,
    ) -> anyhow::Result<()> {
        let num_validators = self.count_vals.unwrap_or(1);

        let mut smoke =
            LibraSmoke::new_with_bundle(Some(num_validators), None, framework_mrb).await?;

        if let Some(p) = twin_db {
            let db_path = fs::canonicalize(p)?;

            make_twin_swarm::awake_frankenswarm(&mut smoke, Some(db_path)).await?;
        } else {
            smoke
                .swarm
                .wait_all_alive(tokio::time::Duration::from_secs(120))
                .await?;
        }

        // NOTE: all validators will stop when the LibraSmoke goes out of context. This is intentional
        // but since it's borrowed in this function you should assume it will continue until the caller goes out of scope.
        dialoguer::Confirm::new()
            .with_prompt("swarm will keep running, until you exit. Press any key to exit.")
            .interact()?;

        Ok(())
    }
}

#[tokio::test]
async fn test_twin_swarm() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(Some(1), None).await?;

    make_twin_swarm::awake_frankenswarm(&mut smoke, None).await?;
    Ok(())
}
