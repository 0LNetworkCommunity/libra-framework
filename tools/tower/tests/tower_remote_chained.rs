

use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use libra_tower::core::{backlog, proof, next_proof};
use libra_smoke_tests::configure_validator;
use libra_types::test_drop_helper::DropTemp;
use libra_types::exports::Client;

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_remote_chained() {
    let d: DropTemp = DropTemp::new_in_crate("temp_smoke_test");

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(1, release).await;


    let (_, app_cfg) = configure_validator::init_val_config_files(&mut swarm, 0, d.dir()).await.expect("could not init validator config");

    // check the tower state is blank
    assert!(backlog::get_remote_tower_height(&app_cfg).await.is_err());

    let _proof = proof::write_genesis(&app_cfg).expect("could not write genesis proof");

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();
    assert!(count == 1);

    let client = Client::new(app_cfg.pick_url(None).unwrap());

    let next = next_proof::get_next_proof_from_chain(&app_cfg, &client).await.expect("could not get proof from chain");
    
    proof::mine_once(&app_cfg, next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 2);

    let next = next_proof::get_next_proof_from_chain(&app_cfg, &client).await.expect("could not get proof from chain");

    proof::mine_once(&app_cfg, next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 3);
}
