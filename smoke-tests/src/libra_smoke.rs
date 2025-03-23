//! The smoke tests should be located in each module (not in the test harness folder), e.g. (tools/txs). This provides wrapper for other modules to import as a dev_dependency. It produces a default swarm with libra configurations and returns the needed types to run tests.

use anyhow::Context;
use diem_crypto::traits::ValidCryptoMaterialStringExt;
use diem_forge::{LocalSwarm, Node, Swarm};
use diem_framework::ReleaseBundle;
use diem_sdk::types::LocalAccount;
use diem_temppath::TempPath;
use diem_types::chain_id::NamedChain;
use libra_framework::release::ReleaseTarget;
use libra_types::core_types::app_cfg::AppCfg;
use libra_types::core_types::network_playlist::NetworkPlaylist;
use libra_types::exports::AccountAddress;
use libra_types::exports::Client;
use smoke_test::smoke_test_environment;
use std::path::PathBuf;
use url::Url;

use crate::helpers;

/// We provide the minimal set of structs to conduct most tests: a swarm object, and a validator keys object (LocalAccount)
pub struct LibraSmoke {
    /// the swarm object
    pub swarm: LocalSwarm,
    /// the first validator account
    pub first_account: LocalAccount,
    /// we often need the encoded private key to test 0L cli tools, so we add it here as a convenience.
    pub encoded_pri_key: String,
    /// Api endpoint
    pub api_endpoint: Url,

    pub validator_private_keys: Vec<String>,
}

// like DropTemp, but tries to make all the nodes stop on drop.
// NOTE: Using drop trait for cleaning up env
// https://doc.rust-lang.org/std/ops/trait.Drop.html
impl Drop for LibraSmoke {
    fn drop(&mut self) {
        println!("libra smoke test dropped, running cleanup");
        let nodes = self.swarm.validators_mut();
        nodes.for_each(|n| n.stop());
    }
}

impl LibraSmoke {
    /// start a swarm and return first val account.
    /// defaults to Head release.
    pub async fn new(count_vals: Option<u8>, libra_bin_path: Option<PathBuf>) -> anyhow::Result<Self> {
        Self::new_with_target(count_vals, libra_bin_path, ReleaseTarget::Head).await
    }
    /// start a swarm and specify the target name e.g. HEAD
    // TODO: deprecate this message,
    // prefer to always pass the file path.
    pub async fn new_with_target(
        count_vals: Option<u8>,
        libra_bin_path: Option<PathBuf>,
        target: ReleaseTarget,
    ) -> anyhow::Result<Self> {
        let release = target.load_bundle().unwrap();
        Self::new_with_bundle(count_vals, libra_bin_path, release).await
    }

    /// start a swarm and specify the release bundle
    pub async fn new_with_bundle(
        count_vals: Option<u8>,
        libra_bin_path: Option<PathBuf>,
        bundle: ReleaseBundle,
    ) -> anyhow::Result<Self> {
        if let Some(p) = libra_bin_path {
            std::env::set_var("DIEM_FORGE_NODE_BIN_PATH", p);
        }

        let diem_path = std::env::var("DIEM_FORGE_NODE_BIN_PATH")
            .expect("DIEM_FORGE_NODE_BIN_PATH not set in env");
        assert!(
            PathBuf::from(&diem_path).exists(),
            "doesn't seem like you have a binary linked to DIEM_FORGE_NODE_BIN_PATH"
        );
        println!("Using diem-node binary at {:?}", &diem_path);

        let mut swarm = smoke_test_environment::new_local_swarm_with_release(
            count_vals.unwrap_or(1).into(),
            bundle,
        )
        .await;

        // First, collect the validator addresses
        let validator_addresses: Vec<_> = swarm.validators().map(|node| node.peer_id()).collect();

        // Initialize an empty Vec to store the private keys
        let mut validator_private_keys = Vec::new();

        // Iterate over the validator addresses
        for &validator_address in &validator_addresses {
            // Create a mutable borrow of `swarm` within the loop to limit its scope
            let mut pub_info = swarm.diem_public_info();
            println!("Minting coins to {:?}", validator_address);

            // Mint and unlock coins
            helpers::mint_libra(&mut pub_info, validator_address, 1000 * 1_000_000)
                .await
                .context("could not mint to account")?;
            helpers::unlock_libra(&mut pub_info, validator_address, 1000 * 1_000_000)
                .await
                .context("could not unlock coins")?;

            // commit note: unsure why we are dropping this
            // Drop the mutable borrow of `swarm` by dropping `pub_info`
            // drop(pub_info);

            // Now it's safe to immutably borrow `swarm`
            let node = swarm.validator(validator_address).unwrap(); // Adjust as needed
            let pri_key = node
                .account_private_key()
                .as_ref()
                .context("no private key for validator")?;
            let encoded_pri_key = pri_key
                .private_key()
                .to_encoded_string()
                .expect("cannot decode pri key");

            // Store the encoded private key
            validator_private_keys.push(encoded_pri_key);
        }

        let node = swarm
            .validators()
            .next()
            .context("no first validator")?
            .to_owned();

        // set up libra_smoke object
        let pri_key = node
            .account_private_key()
            .as_ref()
            .context("no private key for validator")?;
        let encoded_pri_key = pri_key
            .private_key()
            .to_encoded_string()
            .expect("cannot decode pri key");
        let first_account = LocalAccount::new(node.peer_id(), pri_key.private_key(), 0);
        let api_endpoint = node.rest_api_endpoint();

        println!(
            "SUCCESS: swarm started! Use API at: {}",
            api_endpoint.as_str()
        );

        // TODO: order here is awkward because of borrow issues. Clean this up.
        // mint one coin to the main validator.
        // the genesis does NOT mint by default to genesis validators
        // 10,000 coins with 6 decimals precision

        Ok(Self {
            swarm,
            first_account,
            encoded_pri_key,
            api_endpoint,
            validator_private_keys,
        })
    }

    pub async fn mint_and_unlock(
        &mut self,
        addr: AccountAddress,
        amount: u64,
    ) -> anyhow::Result<()> {
        let mut pub_info = self.swarm.diem_public_info();

        helpers::mint_libra(&mut pub_info, addr, amount).await?;
        //helpers::unlock_libra(&mut pub_info, addr, amount).await?;

        Ok(())
    }

    pub fn client(&mut self) -> Client {
        self.swarm.diem_public_info().client().to_owned()
    }

    pub fn marlon_rando(&mut self) -> LocalAccount {
        self.swarm.diem_public_info().random_account()
    }

    pub fn first_account_app_cfg(&mut self) -> anyhow::Result<AppCfg> {
        let config_path = TempPath::new();
        config_path.create_as_dir()?;

        let info = self.swarm.chain_info();
        let chain_name = NamedChain::from_chain_id(&info.chain_id).ok();
        let np = NetworkPlaylist::new(Some(info.rest_api().parse()?), chain_name);
        let mut a = AppCfg::init_app_configs(
            self.first_account.authentication_key(),
            self.first_account.address(),
            Some(config_path.path().into()),
            chain_name,
            Some(np),
        )?;
        let net = a.get_network_profile_mut(None)?;
        net.replace_all_urls(info.rest_api().parse()?);

        let prof = a.get_profile_mut(None)?;
        prof.set_private_key(self.first_account.private_key());
        Ok(a)
    }

    //TODO: Create coin store to be able to fund these accounts
    pub async fn create_accounts(
        &mut self,
        num_accounts: usize,
    ) -> anyhow::Result<(Vec<LocalAccount>, Vec<AccountAddress>)> {
        let mut signers = Vec::new();
        let mut signer_addresses = Vec::new();

        for _ in 0..num_accounts {
            let local_account = self.marlon_rando();
            signer_addresses.push(local_account.address());
            signers.push(local_account);
        }

        Ok((signers, signer_addresses))
    }
}
