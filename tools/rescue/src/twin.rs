use std::{path::PathBuf, time::Duration};

use anyhow::bail;
use clap::Parser;
use diem_types::transaction::Script;

use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::txs_cli_vals::ValidatorTxs;

#[derive(Parser)]

/// set up a twin of the network, with a synced db
/// and replace the validator with local Swarm validators
pub struct TwinOpts {
    // path of snapshot db we want marlon to drive
    #[clap(value_parser)]
    pub db_dir: PathBuf,
    /// The operator.yaml file which contains registration information
    #[clap(value_parser)]
    pub oper_file: Option<PathBuf>,
}

impl TwinOpts {
    /// takes a snapshot db and makes a validator set of ONE from an
    /// existing or NEW marlon rando account
    pub fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// we need a new account config created locally
    async fn initialize_marlon_the_val() -> anyhow::Result<PathBuf> {
        // we use LibraSwarm to create a new folder with validator configs.
        // we then take the operator.yaml, and use it to register on a dirty db

        let mut s = LibraSmoke::new(Some(1)).await?;
        s.swarm.wait_all_alive(Duration::from_secs(10)).await?;
        let marlon = s.swarm.validators_mut().next().unwrap();
        marlon.stop();

        Ok(marlon.config_path().join("operator.yaml"))
    }

    /// create the validator registration entry function payload
    /// needs the file operator.yaml
    fn register_marlon_tx(file: PathBuf) -> anyhow::Result<Script> {
        let tx = ValidatorTxs::Register {
            operator_file: Some(file),
        }
        .make_payload()?
        .encode();
        if let diem_types::transaction::TransactionPayload::Script(s) = tx {
            return Ok(s);
        }
        bail!("function did not return a script")
    }

    /// create the rescue blob which has one validator
    fn recue_blob_with_one_val() {}

    // end to end with rando
    fn apply_with_rando_e2e() {}
}
