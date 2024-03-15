use diem_types::account_address::AccountAddress;

use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBlob {
    /// acccount
    pub account: AccountAddress,
}

impl UserBlob {
    pub fn read(path: PathBuf) -> anyhow::Result<Vec<AccountAddress>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let u: Vec<UserBlob> = serde_json::from_reader(reader)?;
        let list: Vec<AccountAddress> = u.iter().map(|el| el.account).collect();
        Ok(list)
    }
    // helper for clap arg
    pub fn get_vals(path_opt: Option<PathBuf>) -> Option<Vec<AccountAddress>> {
        if let Some(p) = path_opt {
            Self::read(p).ok()
        } else {
            None
        }
    }
}
