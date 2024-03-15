use clap::Parser;
use diem_types::{
    account_address::AccountAddress,
    transaction::{Transaction, WriteSetPayload},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::session_tools;

#[derive(Parser)]

/// Sad day when we must say goodbye
pub struct ForkOpts {
    /// path of snapshot db we want marlon to drive
    #[clap(short, long)]
    pub db_dir: PathBuf,
    /// The operator.yaml file which contains registration information
    #[clap(short, long)]
    pub account_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBlob {
    /// acccount
    account: AccountAddress,
}

impl ForkOpts {
    pub fn run(&self) -> anyhow::Result<PathBuf> {
        let file = File::open(&self.account_file)?;
        let reader = BufReader::new(file);
        let u: Vec<UserBlob> = serde_json::from_reader(reader)?;

        let list: Vec<AccountAddress> = u.iter().map(|el| el.account).collect();

        let cs = session_tools::load_them_onto_ark_b(&self.db_dir, &list)?;
        let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));

        let out = self.db_dir.join("hard_fork.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&out, bytes.as_slice())?;
        Ok(out)
    }
}
