use libra_types::{legacy_types::{app_cfg::AppCfg, network_playlist::NetworkPlaylist}, exports::AuthenticationKey};
use zapatos_sdk::crypto::ValidCryptoMaterialStringExt;
use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};

use libra_framework::release::ReleaseTarget;
use libra_tower::core::{backlog, proof, next_proof};
use zapatos_forge::Swarm;
use libra_types::{test_drop_helper::DropTemp};
use url::Url;
/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_genesis() {
    let d = DropTemp::new_in_crate("temp_smoke_tower");

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(1, release).await;

    let info = swarm.aptos_public_info_for_node(0);
    let url: Url = info.url().parse().unwrap();

    // NetworkPlaylist::default()

    let node = swarm.validators().into_iter().next().unwrap();
    let np = NetworkPlaylist::testing(Some(url));
    let mut app_cfg = AppCfg::init_app_configs(
      AuthenticationKey::ed25519(&node.account_private_key().as_ref().unwrap().public_key()),
      node.peer_id(),
      Some(d.dir()),
      Some(np.chain_id),
      Some(np),
    ).unwrap();

    let pri_key = node.account_private_key().as_ref().expect("could not get pri_key").private_key();
    dbg!(&pri_key.to_encoded_string());
    let profile = app_cfg.get_profile_mut(None).expect("could not get profile");
    profile.test_private_key = Some(pri_key);
    dbg!(&profile);

    // check the tower state is blank
    assert!(backlog::get_remote_tower_height(&app_cfg).await.is_err());

    let _proof = proof::write_genesis(&app_cfg).expect("could not write genesis proof");

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();
    assert!(count == 1);

    let next = next_proof::get_next_proof_params_from_local(&app_cfg).unwrap();
    
    proof::mine_once(&app_cfg, next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 2);

    let next = next_proof::get_next_proof_params_from_local(&app_cfg).unwrap();

    proof::mine_once(&app_cfg, next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 3);
}
