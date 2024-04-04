//! recovery

use crate::exports::AuthenticationKey;

use crate::legacy_types::{
    ancestry_legacy::LegacyAncestryResource,
    cumulative_deposits::{CumulativeDepositResource, LegacyBalanceResource},
    legacy_address::LegacyAddress,
    legacy_currency_info::CurrencyInfoResource,
    legacy_miner_state_v5::TowerStateResourceV5,
    makewhole_resource::MakeWholeResource,
    receipts::ReceiptsResource,
    validator_config::ValidatorConfigResource,
    wallet::{SlowWalletListResource,
    SlowWalletResource},
};





use serde::{Deserialize, Serialize};

use std::{fs, io::Write, path::PathBuf};

use super::legacy_cw_resource_v5::CommunityWalletsResourceLegacyV5;



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Account role
pub enum AccountRole {
    /// System Accounts
    System,
    /// Vals
    Validator,
    /// Opers
    Operator,
    /// Users
    EndUser,
}

impl Default for AccountRole {
    fn default() -> Self {
        Self::EndUser
    }
}

// /// Wallet type
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum WalletType {
//     ///
//     Normal,
//     ///
//     Slow,
//     ///
//     Community,
// }

/// The basic structs needed to recover account state in a new network.
/// This is necessary for catastrophic recoveries, when the source code changes too much.
/// Like what is going to happen between v4 and v5, where the source code of v5
/// will not be able to work with objects from v4. We need an intermediary file.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LegacyRecoveryV5 {
    ///
    pub account: Option<LegacyAddress>,
    ///
    pub auth_key: Option<AuthenticationKey>,
    ///
    pub role: AccountRole,
    ///
    pub balance: Option<LegacyBalanceResource>,
    ///
    pub val_cfg: Option<ValidatorConfigResource>,
    ///
    pub miner_state: Option<TowerStateResourceV5>,
    ///
    pub comm_wallet: Option<CommunityWalletsResourceLegacyV5>,

    pub currency_info: Option<CurrencyInfoResource>,
    ///
    pub ancestry: Option<LegacyAncestryResource>,
    ///
    pub make_whole: Option<MakeWholeResource>,
    ///
    pub receipts: Option<ReceiptsResource>,
    ///
    pub cumulative_deposits: Option<CumulativeDepositResource>,
    ///
    pub slow_wallet: Option<SlowWalletResource>,
    ///
    pub slow_wallet_list: Option<SlowWalletListResource>,
    // TODO: use on V7 tools
    // ///
    // pub fullnode_counter: Option<FullnodeCounterResource>,
    // ///
    // pub autopay: Option<AutoPayResource>,
}

//////// 0L ///////
/// Validator/owner state to recover in genesis recovery mode
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ValStateRecover {
    ///
    pub val_account: LegacyAddress,
    ///
    pub operator_delegated_account: LegacyAddress,
    ///
    pub val_auth_key: AuthenticationKey,
}

//////// 0L ///////
/// Operator state to recover in genesis recovery mode
#[derive(Debug, Clone, PartialEq)]
pub struct OperRecover {
    ///
    pub operator_account: LegacyAddress,
    ///
    pub operator_auth_key: AuthenticationKey,
    ///
    pub validator_to_represent: LegacyAddress,
    ///
    pub operator_consensus_pubkey: Vec<u8>,
    ///
    pub validator_network_addresses: Vec<u8>,
    ///
    pub fullnode_network_addresses: Vec<u8>,
}

/// RecoveryFile
#[derive(Debug, Clone, Default)]
pub struct RecoverConsensusAccounts {
    ///
    pub vals: Vec<ValStateRecover>,
    ///
    pub opers: Vec<OperRecover>,
}

// impl Default for RecoverConsensusAccounts {
//     fn default() -> Self {
//         RecoverConsensusAccounts {
//             vals: vec![],
//             opers: vec![],
//         }
//     }
// }

/// Save genesis recovery file
pub fn save_recovery_file(data: &[LegacyRecoveryV5], path: &PathBuf) -> anyhow::Result<()> {
    let j = serde_json::to_string(data)?;
    let mut file = fs::File::create(path).expect("Could not genesis_recovery create file");
    file.write_all(j.as_bytes())
        .expect("Could not write account recovery");
    Ok(())
}

/// Read from genesis recovery file
pub fn read_from_recovery_file(path: &PathBuf) -> Vec<LegacyRecoveryV5> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str(&data).expect("Unable to parse")
}
