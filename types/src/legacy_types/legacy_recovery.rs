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
use diem_types::validator_config::{ValidatorConfig, ValidatorOperatorConfigResource};
use crate::legacy_types::burn::{BurnCounterResource, UserBurnPreferenceResource};
use crate::legacy_types::donor_voice::RegistryResource;
use crate::legacy_types::donor_voice_txs::{TxScheduleResource};
use crate::legacy_types::fee_maker::{EpochFeeMakerRegistryResource, FeeMakerResource};
use crate::legacy_types::jail::JailResource;
use crate::legacy_types::match_index::MatchIndexResource;
use crate::legacy_types::pledge_account::{MyPledgesResource};
use crate::legacy_types::validator_universe::ValidatorUniverseResource;
use crate::legacy_types::vouch::MyVouchesResource;

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
    pub val_cfg: Option<ValidatorConfig>,
    ///
    pub val_operator_cfg: Option<ValidatorOperatorConfigResource>,
    ///
    pub miner_state: Option<TowerStateResource>,
    ///
    pub comm_wallet: Option<CommunityWalletsResourceLegacy>,

    ///
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

    ///
    pub user_burn_preference: Option<UserBurnPreferenceResource>,

    ///
    pub my_vouches: Option<MyVouchesResource>,

    ///
    pub tx_schedule: Option<TxScheduleResource>,

    ///
    pub fee_maker: Option<FeeMakerResource>,

    ///
    pub jail: Option<JailResource>,

    ///
    pub my_pledge: Option<MyPledgesResource>,

    ///
    pub burn_counter: Option<BurnCounterResource>,

    pub donor_voice_registry: Option<RegistryResource>,

    pub epoch_fee_maker_registry: Option<EpochFeeMakerRegistryResource>,

    pub match_index: Option<MatchIndexResource>,

    pub validator_universe: Option<ValidatorUniverseResource>,

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
        val_operator_cfg: None,
        miner_state: None,
        comm_wallet: None,
        currency_info: None, // TODO: DO WE NEED THIS
        ancestry: None,
        receipts: None,
        cumulative_deposits: None,
        slow_wallet: None,
        slow_wallet_list: None,
        user_burn_preference: None,
        my_vouches: None,
        tx_schedule: None,
        fee_maker: None,
        jail: None,
        my_pledge: None,
        burn_counter: None,
        donor_voice_registry: None,
        epoch_fee_maker_registry: None,
        match_index: None,
        validator_universe: None,
    };
    let account_resource = account_state.get_account_resource()?;

    if let Some(account_resource) = account_resource {
        let byte_slice: [u8; 32] = account_resource.authentication_key()
            .to_vec().try_into().map_err(|err| { anyhow!("error: {:?}", err) })?;

        // auth key
        legacy_recovery.auth_key = Some(AuthenticationKey::new(byte_slice));

        // balance
        // native CoinStoreResource doesn't implement COpy thus use LegacyBalanceResource instead
        legacy_recovery.balance = account_state.get_coin_store_resource()?.map(|r| LegacyBalanceResource {
            coin: r.coin(),
        });

        // validator config
        legacy_recovery.val_cfg = account_state.get_validator_config_resource()?;
        // if let Some(val_cfg) = &legacy_recovery.val_cfg {
        //     println!("val_cfg: {:?}", &val_cfg);
        // }
        legacy_recovery.val_operator_cfg = account_state.get_validator_operator_config_resource()?;
        // if let Some(val_operator_cfg) = &legacy_recovery.val_operator_cfg {
        //     println!("val_operator_cfg: {:?}", &val_operator_cfg);
        // }

        // miner state
        legacy_recovery.miner_state = account_state.get_move_resource::<TowerStateResource>()?;
        // if let Some(miner_state) = &legacy_recovery.miner_state {
        //     println!("miner_state: {:?}", &miner_state);
        // }

        // comm_wallet
        legacy_recovery.comm_wallet = account_state.get_move_resource::<CommunityWalletsResourceLegacy>()?;
        // if let Some(comm_wallet) = &legacy_recovery.comm_wallet {
        //     println!("comm_wallet: {:?}", &comm_wallet);
        // }

        // ancestry
        legacy_recovery.ancestry = account_state.get_move_resource::<LegacyAncestryResource>()?;
        // if let Some(ancestry) = &legacy_recovery.ancestry {
        //     println!("ancestry: {:?}", &ancestry);
        // }

        // receipts
        legacy_recovery.receipts = account_state.get_move_resource::<ReceiptsResource>()?;
        // if let Some(receipts) = &legacy_recovery.receipts {
        //     println!("receipts: {:?}", &receipts);
        // }

        // cumulative_deposits
        legacy_recovery.cumulative_deposits = account_state.get_move_resource::<CumulativeDepositResource>()?;
        // if let Some(cumulative_deposits) = &legacy_recovery.cumulative_deposits {
        //     println!("cumulative_deposits: {:?}", &cumulative_deposits);
        // }

        // slow wallet
        legacy_recovery.slow_wallet = account_state.get_move_resource::<SlowWalletResource>()?;
        // if let Some(slow_wallet) = &legacy_recovery.slow_wallet {
        //     println!("slow_wallet: {:?}", &slow_wallet);
        // }

        // slow wallet list
        legacy_recovery.slow_wallet_list = account_state.get_move_resource::<SlowWalletListResource>()?;
        // if let Some(slow_wallet_list) = &legacy_recovery.slow_wallet_list {
        //     println!("slow_wallet_list: {:?}", &slow_wallet_list);
        // }

        // user burn preference
        // fixtures/state_epoch_79_ver_33217173.795d/0-.chunk has no such users
        legacy_recovery.user_burn_preference = account_state.get_move_resource::<UserBurnPreferenceResource>()?;
        if let Some(user_burn_preference) = &legacy_recovery.user_burn_preference {
            println!("user_burn_preference: {:?}", &user_burn_preference);
        }

        // my vouches
        legacy_recovery.my_vouches = account_state.get_move_resource::<MyVouchesResource>()?;
        // if let Some(my_vouches) = &legacy_recovery.my_vouches {
        //     println!("my_vouches: {:?}", &my_vouches);
        // }

        // tx schedule
        legacy_recovery.tx_schedule = account_state.get_move_resource::<TxScheduleResource>()?;
        // if let Some(tx_schedule) = &legacy_recovery.tx_schedule {
        //     println!("tx_schedule: {:?}", &tx_schedule);
        // }

        // fee maker
        legacy_recovery.fee_maker = account_state.get_move_resource::<FeeMakerResource>()?;
        // if let Some(fee_maker) = &legacy_recovery.fee_maker {
        //     println!("fee_maker: {:?}", &fee_maker);
        // }

        // jail
        legacy_recovery.jail = account_state.get_move_resource::<JailResource>()?;
        // if let Some(jail) = &legacy_recovery.jail {
        //     println!("jail: {:?}", &jail);
        // }

        // pledge account
        legacy_recovery.my_pledge = account_state.get_move_resource::<MyPledgesResource>()?;
        // if let Some(my_pledges) = &legacy_recovery.my_pledge {
        //     println!("my_pledges: {:?}", &my_pledges);
        // }

        // burn counter
        legacy_recovery.burn_counter = account_state.get_move_resource::<BurnCounterResource>()?;
        // if let Some(burn_counter) = &legacy_recovery.burn_counter {
        //     println!("burn_counter: {:?}", &burn_counter);
        // }

        legacy_recovery.donor_voice_registry = account_state.get_move_resource::<RegistryResource>()?;
        // if let Some(donor_voice_registry) = &legacy_recovery.donor_voice_registry {
        //     println!("donor_voice_registry: {:?}", &donor_voice_registry);
        // }

        // epoch fee maker registry
        legacy_recovery.epoch_fee_maker_registry = account_state.get_move_resource::<EpochFeeMakerRegistryResource>()?;
        // if let Some(epoch_fee_maker_registry) = &legacy_recovery.epoch_fee_maker_registry {
        //     println!("epoch_fee_maker_registry: {:?}", &epoch_fee_maker_registry);
        // }

        // match index
        legacy_recovery.match_index = account_state.get_move_resource::<MatchIndexResource>()?;
        // if let Some(match_index) = &legacy_recovery.match_index {
        //     println!("match_index: {:?}", &match_index);
        // }

        // validator universe
        legacy_recovery.validator_universe = account_state.get_move_resource::<ValidatorUniverseResource>()?;
        if let Some(validator_universe) = &legacy_recovery.validator_universe {
            println!("validator_universe: {:?}", &validator_universe);
        }
    }

    Ok(legacy_recovery)
}

