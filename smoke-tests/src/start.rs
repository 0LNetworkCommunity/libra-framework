//! The smoke tests should be located in each module (not in the test harness folder), e.g. (tools/txs). This provides wrapper for other modules to import as a dev_dependency. It produces a default swarm with libra configurations and returns the needed types to run tests.
//!
use anyhow::Context;
use libra_framework::release::ReleaseTarget;
use zapatos_forge::LocalSwarm;
use zapatos_sdk::types::LocalAccount;
use zapatos_smoke_test::smoke_test_environment;
/// We provide the minimal set of structs to conduct most tests: a swarm object, and a validator keys object (LocalAccount)
pub struct LibraSmoke {
  /// the swarm object
  pub swarm: LocalSwarm,
  /// the first validator account
  pub first_account: LocalAccount,
}
impl LibraSmoke {
  /// start a swarm and return first val account
  pub async fn new(count_vals: Option<u8>) -> anyhow::Result<Self> {
      let release = ReleaseTarget::Head.load_bundle().unwrap();
      let mut swarm = smoke_test_environment::new_local_swarm_with_release(count_vals.unwrap_or(1).into(), release).await;

      let v = swarm.validators_mut()
        .next()
        .context("no first validator")?;
      let pri_key = v.account_private_key()
        .as_ref().
        context("no private key for validator")?;

      let first_account = LocalAccount::new(v.peer_id(), pri_key.private_key(), 0);
      Ok(Self {
        swarm,
        first_account,
      })
  }
}
