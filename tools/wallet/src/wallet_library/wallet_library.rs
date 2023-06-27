// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The following document is a minimalist version of Diem Wallet. Note that this Wallet does
//! not promote security as the mnemonic is stored in unencrypted form. In future iterations,
//! we will be releasing more robust Wallet implementations. It is our intention to present a
//! foundation that is simple to understand and incrementally improve the DiemWallet
//! implementation and it's security guarantees throughout testnet. For a more robust wallet
//! reference, the authors suggest to audit the file of the same name in the rust-wallet crate.
//! That file can be found here:
//!
//! https://github.com/rust-bitcoin/rust-wallet/blob/master/wallet/src/walletlibrary.rs

use super::{
    // error::WalletError,
    // io_utils,
    key_factory::{ChildNumber, KeyFactory, Seed},
    mnemonic::Mnemonic,
};
use anyhow::{bail, Result};
use zapatos_types::{
  account_address::AccountAddress,
  transaction::authenticator::AuthenticationKey,
};
use zapatos_crypto::ed25519::Ed25519PrivateKey;
use rand::{rngs::OsRng, Rng};
use std::collections::HashMap;

/// WalletLibrary contains all the information needed to recreate a particular wallet
pub struct WalletLibrary {
    mnemonic: Mnemonic,
    key_factory: KeyFactory,
    addr_map: HashMap<AccountAddress, ChildNumber>,
    key_leaf: ChildNumber,
}

impl WalletLibrary {
    /// Constructor that generates a Mnemonic from OS randomness and subsequently instantiates an
    /// empty WalletLibrary from that Mnemonic
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut rng = OsRng;
        let data: [u8; 32] = rng.gen();
        let mnemonic = Mnemonic::mnemonic(&data).unwrap();
        Self::new_from_mnemonic(mnemonic)
    }

    /// Constructor that instantiates a new WalletLibrary from Mnemonic
    pub fn new_from_mnemonic(mnemonic: Mnemonic) -> Self {
        let seed = Seed::new(&mnemonic, "0L"); //////// 0L ////////
        WalletLibrary {
            mnemonic,
            key_factory: KeyFactory::new(&seed).unwrap(),
            addr_map: HashMap::new(),
            key_leaf: ChildNumber(0),
        }
    }

    /// Function that returns the string representation of the WalletLibrary Mnemonic
    /// NOTE: This is not secure, and in general the mnemonic should be stored in encrypted format
    pub fn mnemonic(&self) -> String {
        self.mnemonic.to_string()
    }

    /// Get the current ChildNumber in u64 format
    pub fn key_leaf(&self) -> u64 {
        self.key_leaf.0
    }

    /// Function that iterates from the current key_leaf until the supplied depth
    pub fn generate_addresses(&mut self, depth: u64) -> Result<()> {
        let current = self.key_leaf.0;
        if current > depth {
           bail!("Addresses already generated up to the supplied depth")
        }
        while self.key_leaf != ChildNumber(depth) {
            let _ = self.new_address();
        }
        Ok(())
    }

    /// Function that allows to get the address of a particular key at a certain ChildNumber
    pub fn new_address_at_child_number(
        &mut self,
        child_number: ChildNumber,
    ) -> Result<AccountAddress> {
        let child = self.key_factory.private_child(child_number)?;
        Ok(child.get_address())
    }

    /// Function that generates a new key and adds it to the addr_map and subsequently returns the
    /// AuthenticationKey associated to the PrivateKey, along with it's ChildNumber
    pub fn new_address(&mut self) -> Result<(AuthenticationKey, ChildNumber)> {
        let child = self.key_factory.private_child(self.key_leaf)?;
        let authentication_key = child.get_authentication_key();
        let old_key_leaf = self.key_leaf;
        self.key_leaf.increment();
        if self
            .addr_map
            .insert(authentication_key.derived_address(), old_key_leaf)
            .is_none()
        {
            Ok((authentication_key, old_key_leaf))
        } else {
            bail!("This address is already in your wallet")
        }
    }

    /// Returns a list of all addresses controlled by this wallet that are currently held by the
    /// addr_map
    pub fn get_addresses(&self) -> Result<Vec<AccountAddress>> {
        let mut ret = Vec::with_capacity(self.addr_map.len());
        let rev_map = self
            .addr_map
            .iter()
            .map(|(&k, &v)| (v.as_ref().to_owned(), k.to_owned()))
            .collect::<HashMap<_, _>>();
        for i in 0..self.addr_map.len() as u64 {
            match rev_map.get(&i) {
                Some(account_address) => {
                    ret.push(*account_address);
                }
                None => {
                    bail!("This address is already in your wallet")
                }
            }
        }
        Ok(ret)
    }

    /// Return private key for an address in the wallet
    pub fn get_private_key(&self, address: &AccountAddress) -> Result<Ed25519PrivateKey> {
        if let Some(child) = self.addr_map.get(address) {
            Ok(self.key_factory.private_child(*child)?.get_private_key())
        } else {
            bail!("missing address")
        }
    }

    /// Return authentication key (AuthenticationKey) for an address in the wallet
    pub fn get_authentication_key(&self, address: &AccountAddress) -> Result<AuthenticationKey> {
        if let Some(child) = self.addr_map.get(address) {
            Ok(self
                .key_factory
                .private_child(*child)?
                .get_authentication_key())
        } else {
            bail!("missing address")
        }
    }

    //////// 0L ////////
    pub fn get_key_factory(&self) -> &KeyFactory{
        &self.key_factory
    }    
}

