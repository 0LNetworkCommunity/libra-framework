//! recovery

use crate::exports::AuthenticationKey;

use crate::legacy_types::{
    ancestry_legacy::LegacyAncestryResource,
    cumulative_deposits::{CumulativeDepositResource, LegacyBalanceResource},
    legacy_address::LegacyAddress,
    legacy_currency_info::CurrencyInfoResource,
    legacy_miner_state::TowerStateResource,
    makewhole_resource::MakeWholeResource,
    receipts::ReceiptsResource,
    validator_config::ValidatorConfigResource,
    wallet::{CommunityWalletsResourceLegacy, SlowWalletListResource, SlowWalletResource},
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
use anyhow::anyhow;
use diem_types::account_state::AccountState;
use move_core_types::account_address::AccountAddress;
use diem_types::account_view::AccountView;

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
pub struct LegacyRecovery {
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
    pub miner_state: Option<TowerStateResource>,
    ///
    pub comm_wallet: Option<CommunityWalletsResourceLegacy>,

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
pub fn save_recovery_file(data: &[LegacyRecovery], path: &PathBuf) -> anyhow::Result<()> {
    let j = serde_json::to_string(data)?;
    let mut file = fs::File::create(path).expect("Could not genesis_recovery create file");
    file.write_all(j.as_bytes())
        .expect("Could not write account recovery");
    Ok(())
}

/// Read from genesis recovery file
pub fn read_from_recovery_file(path: &PathBuf) -> Vec<LegacyRecovery> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str(&data).expect("Unable to parse")
}


use crate::legacy_types::validator_config::ConfigResource;

#[derive(Debug, Default, Clone)]
pub struct LegacyRecoveryV6 {
    ///
    pub account: Option<AccountAddress>,
    ///
    pub auth_key: Option<AuthenticationKey>,
    ///
    pub role: AccountRole,
    ///
    pub balance: Option<LegacyBalanceResource>,
    ///
    pub val_cfg: Option<ValidatorConfigResource>,
    ///
    pub miner_state: Option<TowerStateResource>,
    ///
    pub comm_wallet: Option<CommunityWalletsResourceLegacy>,

    pub currency_info: Option<CurrencyInfoResource>,
    ///
    pub ancestry: Option<LegacyAncestryResource>,
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

pub fn get_legacy_recovery(account_state: &AccountState) -> anyhow::Result<LegacyRecoveryV6> {
    let mut legacy_recovery = LegacyRecoveryV6 {
        account: account_state.get_account_address()?,
        auth_key: None,
        role: AccountRole::EndUser,
        balance: None,
        val_cfg: None,
        miner_state: None,
        comm_wallet: None,
        //fullnode_counter: None,
        //autopay: None,
        currency_info: None,
        ancestry: None,
        receipts: None,
        cumulative_deposits: None,
        slow_wallet: None,
        slow_wallet_list: None,
    };
    let account_resource = account_state.get_account_resource()?;

    if let Some(account_resource) = account_resource {
        let byte_slice: [u8; 32] = account_resource.authentication_key()
            .to_vec().try_into().map_err(|err| { anyhow!("error: {:?}", err) })?;

        // auth key
        legacy_recovery.auth_key = Some(AuthenticationKey::new(byte_slice));

        // balance
        legacy_recovery.balance = account_state.get_coin_store_resource()?.map(|r| LegacyBalanceResource {
            coin: r.coin(),
        });

        // val_cfg
        let validator_config = account_state.get_validator_config_resource()?;
        let validator_operator_config = account_state.get_validator_operator_config_resource()?;


        let validator_config_resource = ValidatorConfigResource {
            config: if let Some(validator_config) = validator_config {
                Some(ConfigResource {
                    consensus_pubkey: Vec::from(validator_config.consensus_public_key.to_bytes()),
                    validator_network_addresses: vec![], // TODO fill out
                    fullnode_network_addresses: validator_config.fullnode_network_addresses.clone(),
                })
            } else {
                None
            },

            operator_account: None, // TODO: account is not available
            human_name: validator_operator_config.map(|r| r.human_name).unwrap_or_else(|| vec![]),
        };

        legacy_recovery.val_cfg = Some(validator_config_resource);

        // miner state
        legacy_recovery.miner_state = account_state.get_resource::<TowerStateResource>()?;
        if let Some(miner_state) = &legacy_recovery.miner_state {
            println!("miner_state: {:?}", &miner_state);
        }

        // comm_wallet
        let comm_wallet = account_state.get_resource::<CommunityWalletsResourceLegacy>()?;
        if let Some(comm_wallet) = &comm_wallet {
            println!("comm_wallet: {:?}", &comm_wallet);
        }
    }

    Ok(legacy_recovery)
}

