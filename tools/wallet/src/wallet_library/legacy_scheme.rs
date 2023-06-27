//! Key derivation for 0L.

use super::{
  key_factory::{ChildNumber, ExtendedPrivKey},
  mnemonic::Mnemonic,
  wallet_library::WalletLibrary,
};
// NOTE: this is included here for compatibility.
// The successor of this struct is KeyChain.

pub struct LegacyKeyScheme {
    /// Owner key, the main key where funds are kept
    pub child_0_owner: ExtendedPrivKey,
    /// Operator of node
    pub child_1_operator: ExtendedPrivKey,
    /// Validator network identity
    pub child_2_val_network: ExtendedPrivKey,
    /// Fullnode network identity
    pub child_3_fullnode_network: ExtendedPrivKey,
    /// Consensus key
    pub child_4_consensus: ExtendedPrivKey,
    /// Execution key
    pub child_5_executor: ExtendedPrivKey,
}

impl LegacyKeyScheme {
    /// Generates the necessary private keys for validator and full node set up.
    pub fn new(wallet: &WalletLibrary) -> Self {
        let kf = wallet.get_key_factory();
        Self {
            child_0_owner: kf.private_child(ChildNumber::new(0)).unwrap(),
            child_1_operator: kf.private_child(ChildNumber::new(1)).unwrap(),
            child_2_val_network: kf.private_child(ChildNumber::new(2)).unwrap(),
            child_3_fullnode_network: kf.private_child(ChildNumber::new(3)).unwrap(),
            child_4_consensus: kf.private_child(ChildNumber::new(4)).unwrap(),
            child_5_executor: kf.private_child(ChildNumber::new(5)).unwrap(),
        }
    }
    /// Get KeyScheme from a mnemonic string.
    pub fn new_from_mnemonic(mnemonic: String) -> LegacyKeyScheme {
        let wallet = WalletLibrary::new_from_mnemonic(Mnemonic::from(&mnemonic).unwrap());
        LegacyKeyScheme::new(&wallet)
    }
}