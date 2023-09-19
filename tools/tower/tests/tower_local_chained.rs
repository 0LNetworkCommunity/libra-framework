//! test tower proof chaining
use diem_temppath::TempPath;
use libra_smoke_tests::configure_validator;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_tower::core::{backlog, next_proof, proof};

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_local_chained() {
    let d = TempPath::new();

    let mut ls = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");

    let (_, app_cfg) =
        configure_validator::init_val_config_files(&mut ls.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    ls.mint(app_cfg.get_profile(None).unwrap().account, 10_000_000)
        .await
        .unwrap();

    // check the tower state is blank
    assert!(backlog::get_remote_tower_height(&app_cfg).await.is_err());

    let _proof = proof::write_genesis(&app_cfg).expect("could not write genesis proof");

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();
    assert!(count == 1);

    let next = next_proof::get_next_proof_params_from_local(&app_cfg).unwrap();

    let path = app_cfg.get_block_dir(None).unwrap();
    proof::mine_once(&path, &next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 2);

    let next = next_proof::get_next_proof_params_from_local(&app_cfg).unwrap();

    let path = app_cfg.get_block_dir(None).unwrap();
    proof::mine_once(&path, &next).unwrap();

    backlog::process_backlog(&app_cfg).await.unwrap();

    let (_proof_num, count) = backlog::get_remote_tower_height(&app_cfg).await.unwrap();

    assert!(count == 3);
}
