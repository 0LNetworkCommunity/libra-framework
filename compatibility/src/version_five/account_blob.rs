// // Copyright (c) The Diem Core Contributors
// // SPDX-License-Identifier: Apache-2.0

use crate::version_five::{
    core_account_v5::AccountResourceV5, diem_account_v5::DiemAccountResourceV5,
    language_v5::StructTagV5, legacy_address_v5::LegacyAddressV5,
};
use anyhow::bail;
use anyhow::{Context, Result};
use diem_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};

use diem_crypto_derive::CryptoHasher;
use move_core_types::language_storage::StructTag;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AccountStateV5(pub BTreeMap<Vec<u8>, Vec<u8>>);

impl AccountStateV5 {
    pub fn get_resource_data<T: MoveResource>(&self) -> Result<&[u8]> {
        // NOTE: don't forget access_vector: has a byte prepended
        let struct_tag = T::struct_tag();

        let legacy_struct_tag = convert_to_legacy(&struct_tag)?;
        let key = legacy_struct_tag.access_vector();

        dbg!(&hex::encode(&key));
        let errmsg = format!(
            "could not find in btree type {}",
            T::struct_tag().to_canonical_string()
        );

        Ok(self.0.get(&key).context(errmsg)?)
    }

    pub fn find_bytes_struct_tag(&self, s: &StructTag) -> Result<&[u8]> {
        let errmsg = format!("could not find in btree type {}", s.to_canonical_string());
        let key = s.access_vector();
        Ok(self.0.get(&key).context(errmsg)?)
    }

    pub fn find_bytes_legacy_struct_tag_v5(
        &self,
        legacy_struct_tag: &StructTagV5,
    ) -> Result<&[u8]> {
        let key = legacy_struct_tag.access_vector();

        dbg!(&hex::encode(&key));
        let errmsg = format!("could not find in btree type {}", legacy_struct_tag.module);

        Ok(self.0.get(&key).context(errmsg)?)
    }

    pub fn get_resource<T: MoveResource>(&self) -> Result<T> {
        let bytes = self.get_resource_data::<T>()?;
        dbg!(&hex::encode(bytes));
        Ok(bcs::from_bytes(bytes)?)
    }

    pub fn get_address(&self) -> Result<LegacyAddressV5> {
        let dr = self.get_diem_account_resource()?;
        Ok(dr.address())
    }

    pub fn get_diem_account_resource(&self) -> Result<DiemAccountResourceV5> {
        self.get_resource::<DiemAccountResourceV5>()
    }

    pub fn get_account_resource(&self) -> Result<AccountResourceV5> {
        match self.get_resource::<AccountResourceV5>() {
            Ok(x) => Ok(x),
            _ => match self.get_resource::<DiemAccountResourceV5>() {
                Ok(diem_ar) => Ok(AccountResourceV5::new(
                    diem_ar.sequence_number(),
                    diem_ar.authentication_key().to_vec(),
                    diem_ar.address(),
                )),
                _ => bail!("can't find an AccountResource or DiemAccountResource"),
            },
        }
    }
}
#[derive(Clone, Eq, PartialEq, Serialize, CryptoHasher)]
pub struct AccountStateBlob {
    pub blob: Vec<u8>,
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

impl CryptoHash for AccountStateBlob {
    type Hasher = AccountStateBlobHasher;

    fn hash(&self) -> HashValue {
        self.hash
    }
}

pub fn convert_to_legacy(s: &StructTag) -> Result<StructTagV5> {
    let legacy_address = LegacyAddressV5::from_hex_literal(&s.address.to_hex_literal())?;

    Ok(StructTagV5 {
        address: legacy_address,
        module: s.module.clone(),
        name: s.name.clone(),
        type_params: vec![], // TODO // s.type_params.clone(),
    })
}
