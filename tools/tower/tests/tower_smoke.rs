use std::path::PathBuf;
use std::str::FromStr;

use libra_types::{legacy_types::app_cfg::AppCfg, exports::AuthenticationKey};
use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use libra_tower::core::proof;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_genesis() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let swarm = new_local_swarm_with_release(4, release).await;

    let node = swarm.validators().into_iter().next().unwrap();
    // let local = LocalAccount::new(node.peer_id(), node.account_private_key().unwrap().private_key(), 0);
    // let info = swarm.aptos_public_info();

    let mut app_cfg = AppCfg::init_app_configs(
      AuthenticationKey::ed25519(&node.account_private_key().as_ref().unwrap().public_key()),
      node.peer_id(),
      None,
      None,
    ).unwrap();
    dbg!(&app_cfg.profile);

    let temp_home_path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    temp_home_path.push("_smoke_test_temp");

    app_cfg.workspace.node_home = temp_home_path;

    std::fs::create_dir_all(temp_home_path);

    let p = proof::write_genesis(&app_cfg).expect("could not write genesis proof");

    dbg!(&p);
// next_proof::get_next_proof_params_from_local(config)?
    // dbg!(&path);
    // libra_tower::

}
