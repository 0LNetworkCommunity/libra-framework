use anyhow::{bail, Result};
use std::path::PathBuf;

use diem_genesis::config::{HostAndPort, OperatorConfiguration, OwnerConfiguration};
use libra_types::global_config_dir;

use crate::{
    keys::PUBLIC_KEYS_FILE,
    utils::{
        from_yaml, read_from_file, read_public_identity_file, to_yaml, write_to_user_only_file,
    },
};

pub const OPERATOR_FILE: &str = "operator.yaml";
pub const OWNER_FILE: &str = "owner.yaml";

// copied from crate/aptos/src/genesis/keys.rs
pub struct SetValidatorConfiguration {
    /// Name of the validator
    pub username: String,

    /// Host and port pair for the validator e.g. 127.0.0.1:6180 or aptoslabs.com:6180
    pub validator_host: HostAndPort,

    /// Host and port pair for the fullnode e.g. 127.0.0.1:6180 or aptoslabs.com:6180
    pub full_node_host: Option<HostAndPort>,

    /// Path to private identity generated from GenerateKeys
    pub home_dir: Option<PathBuf>,
}

impl SetValidatorConfiguration {
    pub fn new(
        home_dir: Option<PathBuf>,
        username: String,
        validator_host: HostAndPort,
        full_node_host: Option<HostAndPort>,
    ) -> Self {
        Self {
            username,
            validator_host,
            full_node_host,
            home_dir,
        }
    }

    pub fn set_config_files(self) -> Result<(OperatorConfiguration, OwnerConfiguration)> {
        let home_dir = self.home_dir.unwrap_or_else(global_config_dir);

        let owner_keys_file = home_dir.join(PUBLIC_KEYS_FILE);

        let owner_identity = read_public_identity_file(owner_keys_file.as_path())?;

        let voter_identity = owner_identity.clone();
        let operator_identity = owner_identity.clone();
        let operator_keys_file = owner_keys_file;

        // Extract the possible optional fields
        let consensus_public_key =
            if let Some(consensus_public_key) = operator_identity.consensus_public_key {
                consensus_public_key
            } else {
                bail!(
                    "Failed to read consensus public key from public identity file {}",
                    operator_keys_file.display()
                );
            };

        let validator_network_public_key = if let Some(validator_network_public_key) =
            operator_identity.validator_network_public_key
        {
            validator_network_public_key
        } else {
            bail!(
                "Failed to read validator network public key from public identity file {}",
                operator_keys_file.display()
            );
        };

        let consensus_proof_of_possession = if let Some(consensus_proof_of_possession) =
            operator_identity.consensus_proof_of_possession
        {
            consensus_proof_of_possession
        } else {
            bail!(
                "Failed to read consensus proof of possession from public identity file {}",
                operator_keys_file.display()
            );
        };

        // Only add the public key if there is a full node
        let full_node_network_public_key = if self.full_node_host.is_some() {
            operator_identity.full_node_network_public_key
        } else {
            None
        };

        // Build operator configuration file
        let operator_config = OperatorConfiguration {
            operator_account_address: operator_identity.account_address.into(),
            operator_account_public_key: operator_identity.account_public_key.clone(),
            consensus_public_key,
            consensus_proof_of_possession,
            validator_network_public_key,
            validator_host: self.validator_host,
            full_node_network_public_key,
            full_node_host: self.full_node_host,
        };

        let owner_config = OwnerConfiguration {
            owner_account_address: owner_identity.account_address.into(),
            owner_account_public_key: owner_identity.account_public_key,
            voter_account_address: voter_identity.account_address.into(),
            voter_account_public_key: voter_identity.account_public_key,
            operator_account_address: operator_identity.account_address.into(),
            operator_account_public_key: operator_identity.account_public_key,
            stake_amount: 100_000_000_000_000,
            commission_percentage: 0,
            join_during_genesis: true,
        };

        write_to_user_only_file(
            &home_dir.join(OPERATOR_FILE),
            OPERATOR_FILE,
            to_yaml(&operator_config)?.as_bytes(),
        )?;

        write_to_user_only_file(
            &home_dir.join(OWNER_FILE),
            OWNER_FILE,
            to_yaml(&owner_config)?.as_bytes(),
        )?;

        Ok((operator_config, owner_config))
    }

    pub fn read_configs_from_file(
        home_path: Option<PathBuf>,
    ) -> Result<(OperatorConfiguration, OwnerConfiguration)> {
        let dir = home_path.unwrap_or_else(global_config_dir);

        let operator_config: OperatorConfiguration = from_yaml(
            &String::from_utf8(read_from_file(&dir.join(OPERATOR_FILE)).unwrap()).unwrap(),
        )
        .unwrap();

        let owner_config: OwnerConfiguration =
            from_yaml(&String::from_utf8(read_from_file(&dir.join(OWNER_FILE)).unwrap()).unwrap())
                .unwrap();

        Ok((operator_config, owner_config))
    }
}
