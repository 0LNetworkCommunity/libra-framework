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
    pub db_path: PathBuf,
    /// The operator.yaml file which contains registration information
    #[clap(short, long)]
    pub account_file: PathBuf,
    /// list of validators in validator set
    #[clap(short, long)]
    pub debug_vals: Option<Vec<AccountAddress>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBlob {
    /// acccount
    account: AccountAddress,
}

impl ForkOpts {
    pub fn run_ark_b(&self) -> anyhow::Result<PathBuf> {
        println!("\"Exciting isn't it?\" said the Captain.");

        let file = File::open(&self.account_file)?;
        let reader = BufReader::new(file);
        let u: Vec<UserBlob> = serde_json::from_reader(reader)?;

        let list: Vec<AccountAddress> = u.iter().map(|el| el.account).collect();

        let cs = session_tools::load_them_onto_ark_b(&self.db_path, &list, self.debug_vals.clone())?;
        let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));

        let out = self.db_path.join("hard_fork.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&out, bytes.as_slice())?;
        println!("\"Ah yes, that was it,\" beamed the Captain, \"that was the reason.\"");

        Ok(out)
    }
}
