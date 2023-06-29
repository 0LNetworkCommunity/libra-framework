use libra_types::{legacy_types::app_cfg::AppCfg, exports::AuthenticationKey};
use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};

use libra_framework::release::ReleaseTarget;
use libra_tower::core::{backlog, proof, next_proof};
use zapatos_forge::Swarm;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_genesis() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(1, release).await;

    let info = swarm.aptos_public_info_for_node(0);
    let _url = info.url().to_string();

    let node = swarm.validators().into_iter().next().unwrap();

    // let local = LocalAccount::new(node.peer_id(), node.account_private_key().unwrap().private_key(), 0);
    // let info = swarm.aptos_public_info();
    
    // let port = swarm.validators().next().unwrap().port();
    // let url = Url::from_str(&format!("http://localhost:{}", port));

    let app_cfg = AppCfg::init_app_configs(
      AuthenticationKey::ed25519(&node.account_private_key().as_ref().unwrap().public_key()),
      node.peer_id(),
      None,
      None,
    ).unwrap();



    // let temp_files = &DropTemp::new_in_crate("_smoke_test_temp").0;

    // app_cfg.workspace.node_home = temp_files.to_owned();
    // app_cfg.profile.upstream_nodes = vec![url.parse().unwrap()];
    // app_cfg.profile.test_private_key = Some(node.account_private_key().as_ref().unwrap().private_key());

    // check the tower state is blank
    assert!(backlog::get_remote_tower_height(&app_cfg).await.is_err());

    let _proof = proof::write_genesis(&app_cfg).expect("could not write genesis proof");

    // dbg!(&proof);

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();
    assert!(count == 1);

    let next = next_proof::get_next_proof_params_from_local(&app_cfg).unwrap();
    
    proof::mine_once(&app_cfg, next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 2);

}
