//! read-archive
use anyhow::{anyhow, Error, Result};
use diem_backup_cli::utils::read_record_bytes::ReadRecordBytes;
use diem_backup_cli::{
    backup_types::{
        epoch_ending::manifest::EpochEndingBackup, state_snapshot::manifest::StateSnapshotBackup,
    },
    storage::{FileHandle, FileHandleRef},
};
use tokio::{fs::OpenOptions, io::AsyncRead};
// use serde_json::json;
// use diem_crypto::HashValue;
// use diem_types::account_state_blob::AccountStateBlob;
use std::{
    fs,
    path::{Path, PathBuf},
};
use std::collections::HashMap;
use diem_types::account_address::AccountAddress;
use diem_types::account_state::AccountState;
use diem_types::account_view::AccountView;
use diem_types::state_store::state_key::{StateKey, StateKeyInner};
use diem_types::state_store::state_key::StateKeyTag::TableItem;
use diem_types::state_store::state_value::StateValue;
use diem_types::transaction::authenticator::AuthenticationKey;
// use tokio::{fs::OpenOptions, io::AsyncRead};

////// SNAPSHOT FILE IO //////
/// read snapshot manifest file into object
pub fn load_snapshot_manifest(path: &PathBuf) -> Result<StateSnapshotBackup, Error> {
    let config = std::fs::read_to_string(path).map_err(|e| {
        format!("Error: cannot read file {:?}, error: {:?}", &path, &e);
        e
    })?;

    let map: StateSnapshotBackup = serde_json::from_str(&config)?;

    Ok(map)
}

/// get the epoch manifest from file
/// NOTE: this file may sometimes be gzipped. So if it's not parsing, that may
/// be why.
// https://github.com/0LNetworkCommunity/diem/blob/restore-hard-forks/storage/backup/backup-cli/src/backup_types/state_snapshot/backup.rs#L260
pub fn load_epoch_manifest(p: &Path) -> Result<EpochEndingBackup> {
    let bytes = fs::read(p)?;
    Ok(serde_json::from_slice::<EpochEndingBackup>(&bytes)?)
}

/// parse each chunk of a state snapshot manifest
pub async fn read_account_state_chunk(
    file_handle: FileHandle,
    archive_path: &Path,
) -> Result<Vec<(StateKey, StateValue)>, Error> {
    let full_handle = archive_path
        .parent()
        .expect("could not read archive path")
        .join(file_handle);
    let handle_str = full_handle.to_str().unwrap();
    let mut file = open_for_read(handle_str)
        .await
        .map_err(|e| anyhow!("snapshot chunk {:?}, {:?}", &handle_str, e))?;

    let mut chunk = vec![];

    while let Some(record_bytes) = file.read_record_bytes().await? {
        //println!("record_bytes: {:?}", &record_bytes.len());
        chunk.push(bcs::from_bytes(&record_bytes)?);
    }
    //println!("chunk: {:?}", &chunk.len()    );
    Ok(chunk)
}

async fn open_for_read(file_handle: &FileHandleRef) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
    let file = OpenOptions::new().read(true).open(file_handle).await?;
    Ok(Box::new(file))
}

/// Tokio async parsing of state snapshot into blob
async fn accounts_from_snapshot_backup(
    manifest: StateSnapshotBackup,
    archive_path: &Path,
) -> anyhow::Result<Vec<AccountState>> {
    // parse AccountStateBlob from chunks of the archive
    let mut account_states_map: HashMap<AccountAddress, Vec<(StateKey, StateValue)>> = HashMap::new();
    for chunk in manifest.chunks {
        let blobs: Vec<(StateKey, StateValue)> = read_account_state_chunk(chunk.blobs, archive_path).await?;

        // Filter out the AccountState blobs
        // and group by address
        let account_states_map_chunk: HashMap<AccountAddress, Vec<(StateKey, StateValue)>> = blobs.into_iter().fold(HashMap::new(), |mut acc, (key, blob)| {
            match key.inner() {
                StateKeyInner::AccessPath(access_path) => {
                    acc.entry(access_path.address).or_insert(vec![]).push((key, blob));
                }
                _ => (),
            }
            acc
        });

        // merge account_states_map_chunk into account_states_map
        for (address, blobs) in account_states_map_chunk {
            account_states_map.entry(address).or_insert(vec![]).extend(blobs);
        }
    }
    // materialize account state for each address
    let mut account_states: Vec<AccountState> = Vec::new();
    for (address, blobs) in account_states_map {
        let blobs_hash_table = blobs.into_iter().fold(HashMap::new(), |mut acc, (key, blob)| {
            acc.insert(key, blob);
            acc
        });
        match AccountState::from_access_paths_and_values(address, &blobs_hash_table)? {
            None => {}
            Some(account_state) => { account_states.push(account_state); }
        };
    }

    Ok(account_states)
}

#[test]
fn test_parse_manifest() {
    use std::str::FromStr;
    let mut this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_path.push("fixtures/state_epoch_79_ver_33217173.795d/state.manifest");
    let r = load_snapshot_manifest(&this_path).expect("parse manifest");
    dbg!(&r.epoch);
}

#[tokio::test]
async fn test_deserialize_account() {
    use std::str::FromStr;
    let mut this_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    this_path.push("fixtures/state_epoch_79_ver_33217173.795d/state.manifest");
    let snapshot_manifest = load_snapshot_manifest(&this_path).expect("parse manifest");
    let archive_path = this_path.parent().unwrap();
    let account_states = accounts_from_snapshot_backup(snapshot_manifest, &archive_path)
        .await
        .expect("could not parse snapshot");
    for account_state in account_states.iter() {
        println!("account_address: {:?}", account_state.get_account_address());
        let r = account_state.get_coin_store_resource();
        println!("coin store: {:?}", r);
    }
}

use libra_types::legacy_types::legacy_recovery::{AccountRole};
use diem_types::validator_config::ValidatorConfig;

use libra_types::legacy_types::{
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

use std::convert::TryInto;
use diem_types::account_config::AccountResource;
use libra_types::legacy_types::validator_config::ConfigResource;

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


fn get_legacy_recovery(account_states: &Vec<AccountState>) -> anyhow::Result<Vec<LegacyRecoveryV6>> {
    let mut legacy_recovery_v6 = Vec::new();
    for account_state in account_states.iter() {
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
            make_whole: None,
            receipts: None,
            cumulative_deposits: None,
            slow_wallet: None,
            slow_wallet_list: None,
        };
        let account_resource = account_state.get_account_resource()?;

        use std::convert::TryInto;
        if let Some(account_resource) = account_resource {
            let byte_slice: [u8; 32] = account_resource.authentication_key()
                .to_vec().try_into().map_err(|err| { anyhow!("error: {:?}", err) })?;

            legacy_recovery.auth_key = Some(AuthenticationKey::new(byte_slice));
            legacy_recovery.balance = account_state.get_coin_store_resource()?.map(|r| LegacyBalanceResource {
                coin: r.coin(),
            });

            let validator_config = account_state.get_validator_config_resource()?;
            let validator_operator_config = account_state.get_validator_operator_config_resource()?;

            let validator_config_resource = ValidatorConfigResource {
                config: if let Some(validator_config) = validator_config {
                    Some(ConfigResource {
                        consensus_pubkey: Vec::from(validator_config.consensus_public_key.to_bytes()),
                        validator_network_addresses: vec![],
                        fullnode_network_addresses: validator_config.fullnode_network_addresses.clone(),
                    })
                } else {
                    None
                },

                operator_account: None, // TODO: account is not available
                human_name: validator_operator_config.map(|r| r.human_name).unwrap_or_else(|| vec![]),
            };

            legacy_recovery.val_cfg = Some(validator_config_resource);
        }

        legacy_recovery_v6.push(legacy_recovery);
        println!("account_address: {:?}", account_state.get_account_address());
        let r = account_state.get_coin_store_resource();
        println!("coin store: {:?}", r);
    }
    Ok(legacy_recovery_v6)
}
