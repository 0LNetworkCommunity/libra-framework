// // Copyright (c) The Diem Core Contributors
// // SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use diem_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};

use std::collections::BTreeMap;
use diem_crypto_derive::CryptoHasher;
use serde::{Deserialize, Deserializer, Serialize};
use std::{convert::TryFrom, fmt};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AccountStateV5(BTreeMap<Vec<u8>, Vec<u8>>);

#[derive(Clone, Eq, PartialEq, Serialize, CryptoHasher)]
pub struct AccountStateBlob {
    blob: Vec<u8>,
    #[serde(skip)]
    hash: HashValue,
}

impl<'de> Deserialize<'de> for AccountStateBlob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "AccountStateBlob")]
        struct RawBlob {
            blob: Vec<u8>,
        }
        let blob = RawBlob::deserialize(deserializer)?;

        Ok(Self::new(blob.blob))
    }
}

impl AccountStateBlob {
    fn new(blob: Vec<u8>) -> Self {
        let mut hasher = AccountStateBlobHasher::default();
        hasher.update(&blob);
        let hash = hasher.finish();
        Self { blob, hash }
    }
}

impl fmt::Debug for AccountStateBlob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decoded = bcs::from_bytes(&self.blob)
            .map(|account_state: AccountStateV5| format!("{:#?}", account_state))
            .unwrap_or_else(|_| String::from("[fail]"));

        write!(
            f,
            "AccountStateBlob {{ \n \
             Raw: 0x{} \n \
             Decoded: {} \n \
             }}",
            hex::encode(&self.blob),
            decoded,
        )
    }
}

impl TryFrom<&AccountStateBlob> for AccountStateV5 {
    type Error = Error;

    fn try_from(account_state_blob: &AccountStateBlob) -> Result<Self> {
        bcs::from_bytes(&account_state_blob.blob).map_err(Into::into)
    }
}


impl CryptoHash for AccountStateBlob {
    type Hasher = AccountStateBlobHasher;

    fn hash(&self) -> HashValue {
        self.hash
    }
}
