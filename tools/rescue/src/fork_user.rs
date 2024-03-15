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
    /// path of DB which will be used to start new network
    #[clap(short, long)]
    pub db_dir: PathBuf,
    /// JSON file with list of accounts to drop on new network
    #[clap(short, long)]
    pub account_file: PathBuf,
    /// optional, JSON file with list of new validators. Must already have on-chain configurations
    #[clap(short, long)]
    pub validators_file: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserBlob {
    /// acccount
    account: AccountAddress,
}

impl ForkOpts {
    pub fn run_ark_b(&self) -> anyhow::Result<PathBuf> {
        println!("\"Exciting isn't it?\" said the Captain.");

        // accounts that will drop
        let file = File::open(&self.account_file)?;
        let reader = BufReader::new(file);
        let u: Vec<UserBlob> = serde_json::from_reader(reader)?;
        let drop_list: Vec<AccountAddress> = u.iter().map(|el| el.account).collect();
        // new validator set
        let vals = if let Some(path) = &self.validators_file {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let u: Vec<UserBlob> = serde_json::from_reader(reader)?;

            let list: Vec<AccountAddress> = u.iter().map(|el| el.account).collect();
            Some(list)
        } else {
          None
        };

        let cs =
            session_tools::load_them_onto_ark_b(&self.db_dir, &drop_list, vals)?;
        let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));

        let out = self.db_dir.join("hard_fork.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&out, bytes.as_slice())?;
        println!("\"Ah yes, that was it,\" beamed the Captain, \"that was the reason.\"");

        Ok(out)
    }
}
