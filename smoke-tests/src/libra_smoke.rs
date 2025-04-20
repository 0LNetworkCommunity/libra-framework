//! The smoke tests should be located in each module (not in the test harness folder), e.g. (tools/txs). This provides wrapper for other modules to import as a dev_dependency. It produces a default swarm with libra configurations and returns the needed types to run tests.

use anyhow::{anyhow, Context};
use diem_crypto::traits::ValidCryptoMaterialStringExt;
use diem_forge::SwarmExt;
use diem_forge::{LocalSwarm, Node, Swarm};
use diem_framework::ReleaseBundle;
use diem_logger::info;
use diem_sdk::types::LocalAccount;
use diem_temppath::TempPath;
use diem_types::chain_id::NamedChain;
use libra_framework::release::ReleaseTarget;
use libra_types::core_types::app_cfg::AppCfg;
use libra_types::core_types::network_playlist::NetworkPlaylist;
use libra_types::exports::Client;
use libra_types::exports::{AccountAddress, AuthenticationKey};
use smoke_test::smoke_test_environment;
use smoke_test::test_utils::MAX_CATCH_UP_WAIT_SECS;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

use crate::helpers::{self, update_node_config_restart, wait_for_node};

/// We provide the minimal set of structs to conduct most tests: a swarm object, and a validator keys object (LocalAccount)
pub struct LibraSmoke {
    /// the swarm object
    pub swarm: LocalSwarm,
    /// the first validator account
    pub first_account: LocalAccount, // TODO: do we use this?
    /// we often need the encoded private key to test 0L cli tools, so we add it here as a convenience.
    pub encoded_pri_key: String, // TODO: do we use this?
    /// An api endpoint, of the first validator
    pub api_endpoint: Url,
    /// A list of the private keys of the validators
    pub validator_private_keys: Vec<String>, // TODO: Dd we use it?
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
    pub async fn new(
        count_vals: Option<u8>,
        libra_bin_path: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
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
        let chain_name =
            NamedChain::from_chain_id(&swarm.chain_info().chain_id).map_err(|e| anyhow!(e))?;
        // First, collect the validator addresses
        let mut validator_addresses: Vec<AccountAddress> = vec![];

        // Initialize an empty Vec to store the private keys
        let mut validator_private_keys = Vec::new();

        // Iterate over the validator addresses
        for local_node in swarm.validators() {
            let v_addr = local_node.peer_id();
            validator_addresses.push(v_addr.to_owned());
            // Create a mutable borrow of `swarm` within the loop to limit its scope

            let pri_key = local_node
                .account_private_key()
                .as_ref()
                .context("no private key for validator")?;
            let encoded_pri_key = pri_key
                .private_key()
                .to_encoded_string()
                .expect("cannot decode pri key");

            // Store the encoded private key
            validator_private_keys.push(encoded_pri_key);

            // now create the appCfg
            let mut app_cfg = AppCfg::init_app_configs(
                AuthenticationKey::from_str(v_addr.to_string().as_str())?, // TODO: these should be the same at the swarm start.
                v_addr.to_owned(),
                Some(local_node.config_path().parent().unwrap().to_path_buf()),
                Some(chain_name),
                Some(NetworkPlaylist::new(
                    Some(local_node.rest_api_endpoint()),
                    Some(chain_name),
                )),
            )?;

            // Sets private key to file
            // DANGER: this is only for testnet
            app_cfg
                .get_profile_mut(None)
                .unwrap()
                .set_private_key(&pri_key.private_key());
            app_cfg.save_file()?;
        }

        // mint to each
        let mut pub_info = swarm.diem_public_info();

        for v_addr in validator_addresses {
            // Mint and unlock coins
            info!("Minting coins to {:?}", &v_addr);
            helpers::mint_libra(&mut pub_info, v_addr, 1000 * 1_000_000)
                .await
                .context("could not mint to account")?;
            helpers::unlock_libra(&mut pub_info, v_addr, 1000 * 1_000_000)
                .await
                .context("could not unlock coins")?;
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
    pub async fn test_setup_start_then_pause(num_nodes: u8) -> anyhow::Result<Self> {
        let mut s = LibraSmoke::new(Some(num_nodes), None)
            .await
            .expect("could not start libra smoke");

        let env = &mut s.swarm;

        env.wait_for_all_nodes_to_catchup_to_version(
            10,
            std::time::Duration::from_secs(MAX_CATCH_UP_WAIT_SECS),
        )
        .await
        .unwrap();

        println!("1. Set sync_only = true for all nodes and restart");
        for node in env.validators_mut() {
            let mut node_config = node.config().clone();
            node_config.consensus.sync_only = true;
            update_node_config_restart(node, &mut node_config)?;
            wait_for_node(node, (num_nodes - 1) as usize).await?;
        }

        println!("2. verify all nodes are at the same round and no progress being made");
        env.wait_for_all_nodes_to_catchup(std::time::Duration::from_secs(MAX_CATCH_UP_WAIT_SECS))
            .await
            .unwrap();

        println!("3. stop nodes");

        for node in env.validators_mut() {
            node.stop();
        }
        Ok(s)
    }
}

#[tokio::test]
async fn meta_can_add_random_vals() -> anyhow::Result<()> {
    let s = LibraSmoke::test_setup_start_then_pause(2).await?;

    crate::helpers::make_test_randos(&s).await?;

    Ok(())
}

#[tokio::test]
/// This meta test checks that our tools can control a network
/// so the nodes stop producing blocks, shut down, and start again.
async fn test_swarm_can_halt_and_restart() -> anyhow::Result<()> {
    use diem_forge::NodeExt;
    let mut s = LibraSmoke::test_setup_start_then_pause(3).await?;

    for node in s.swarm.validators_mut().take(3) {
        assert!(node.liveness_check(1).await.is_err());
    }

    Ok(())
}
