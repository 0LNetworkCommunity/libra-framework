use clap::Parser;
use diem_types::transaction::{Transaction, WriteSetPayload};
use std::path::PathBuf;

use crate::{session_tools, user_file::UserBlob};

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
    /// optional, short epochs for debugging
    #[clap(short, long)]
    pub debug_short_epochs: bool,
}

impl ForkOpts {
    pub fn run_ark_b(&self) -> anyhow::Result<PathBuf> {
        println!("\"Exciting isn't it?\" said the Captain.");

        // accounts that will drop
        let drop_list = UserBlob::get_vals(Some(self.account_file.clone()))
            .expect("missing list of accounts to drop");
        // new validator set
        let vals = UserBlob::get_vals(self.validators_file.clone());

        let cs = session_tools::load_them_onto_ark_b(
            &self.db_dir,
            &drop_list,
            vals,
            self.debug_short_epochs,
        )?;
        let gen_tx = Transaction::GenesisTransaction(WriteSetPayload::Direct(cs));

        let out = self.db_dir.join("hard_fork.blob");

        let bytes = bcs::to_bytes(&gen_tx)?;
        std::fs::write(&out, bytes.as_slice())?;
        println!("\"Ah yes, that was it,\" beamed the Captain, \"that was the reason.\"");

        Ok(out)
    }
}
