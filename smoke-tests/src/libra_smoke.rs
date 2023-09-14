//! The smoke tests should be located in each module (not in the test harness folder), e.g. (tools/txs). This provides wrapper for other modules to import as a dev_dependency. It produces a default swarm with libra configurations and returns the needed types to run tests.

use anyhow::Context;
use diem_crypto::traits::ValidCryptoMaterialStringExt;
use diem_forge::{LocalSwarm, Node, Swarm};
use diem_sdk::types::LocalAccount;
use libra_framework::release::ReleaseTarget;
use libra_types::exports::AccountAddress;
use libra_types::exports::Client;
use smoke_test::smoke_test_environment;
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
}

// like DropTemp, but tries to make all the nodes stop on drop.
// NOTE: Useing drop trait for cleaning up env
// https://doc.rust-lang.org/std/ops/trait.Drop.html
impl Drop for LibraSmoke {
    fn drop(&mut self) {
        println!("test dropped, running cleanup");
        let nodes = self.swarm.validators_mut();
        nodes.for_each(|n| n.stop());
    }
}

impl LibraSmoke {
    /// start a swarm and return first val account
    pub async fn new(count_vals: Option<u8>) -> anyhow::Result<Self> {
        let release = ReleaseTarget::Head.load_bundle().unwrap();
        let mut swarm = smoke_test_environment::new_local_swarm_with_release(
            count_vals.unwrap_or(1).into(),
            release,
        )
        .await;

        let node = swarm
            .validators()
            .next()
            .context("no first validator")?
            .to_owned();
        let addr = node.peer_id();

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

        // TODO: order here is awkward because of borrow issues. Clean this up.
        // mint one coin to the main validator.
        // the genesis does NOT mint by default to genesis validators
        // 10,000 coins with 6 decimals precision
        let mut pub_info = swarm.diem_public_info();
        helpers::mint_libra(&mut pub_info, addr, 100_000_000_000).await?;

        Ok(Self {
            swarm,
            first_account,
            encoded_pri_key,
            api_endpoint,
        })
    }

    pub async fn mint(&mut self, addr: AccountAddress, amount: u64) -> anyhow::Result<()> {
        let mut pub_info = self.swarm.diem_public_info();

        helpers::mint_libra(&mut pub_info, addr, amount).await?;
        Ok(())
    }

    pub fn client(&mut self) -> Client {
        self.swarm.diem_public_info().client().to_owned()
    }

    pub fn marlon_rando(&mut self) -> LocalAccount {
        self.swarm.diem_public_info().random_account()
    }
}
