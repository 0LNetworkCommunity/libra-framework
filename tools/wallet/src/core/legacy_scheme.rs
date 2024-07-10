//! Key derivation for 0L.

use super::{
    key_factory::{ChildNumber, ExtendedPrivKey},
    mnemonic::Mnemonic,
    wallet_library::WalletLibrary,
};
#[cfg(test)]
use diem_crypto::Length;

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

// Verifies that the LegacyKeyScheme struct correctly generates non-empty private keys.
//
// This test suite ensures that the LegacyKeyScheme struct functions as intended by
// generating private keys from a mnemonic and confirming that each generated key (owner,
// operator, network identities, consensus, and executor keys) is not empty. This validates
// that the key derivation process is functioning correctly.
#[test]
fn test_legacy_key_scheme() {
    // Generate a mnemonic for testing purposes
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let key_scheme = LegacyKeyScheme::new_from_mnemonic(mnemonic.to_string());

    // Check that the generated keys are not empty
    assert!(key_scheme.child_0_owner.get_private_key().length() > 0);
    assert!(key_scheme.child_1_operator.get_private_key().length() > 0);
    assert!(key_scheme.child_2_val_network.get_private_key().length() > 0);
    assert!(
        key_scheme
            .child_3_fullnode_network
            .get_private_key()
            .length()
            > 0
    );
    assert!(key_scheme.child_4_consensus.get_private_key().length() > 0);
    assert!(key_scheme.child_5_executor.get_private_key().length() > 0);
}
