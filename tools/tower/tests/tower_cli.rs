use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};

use libra_tower::{
    core::{backlog, next_proof},
    tower_cli::{TowerCli, TowerSub},
};

use libra_types::exports::ValidCryptoMaterialStringExt;

// Scenario: We want to start from a blank slate and with the CLI tool:
// 1. have the validator reate a zeroth proof locally
// 2. Submit that proof to chain.
// 3. Continue mining from the previous proof.
// 4. Successfully resume, after nuking all local proofs.
// 5. Continue mining after a new epoch has started. // TODO

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_cli_e2e() {
    let d = diem_temppath::TempPath::new();

    let mut ls = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");

    let (_, app_cfg) =
        configure_validator::init_val_config_files(&mut ls.swarm, 0, d.path().to_owned())
            .await
            .expect("could not init validator config");

    // check the tower state is blank
    assert!(backlog::get_remote_tower_height(&app_cfg).await.is_err());

    // 1. have the validator reate a zeroth proof locally
    let profile = app_cfg.get_profile(None).unwrap();
    let pri_key_string = profile
        .borrow_private_key()
        .unwrap()
        .to_encoded_string()
        .unwrap();
    let mut cli = TowerCli {
        command: TowerSub::Zero,
        config_file: Some(d.path().join("libra.yaml")),
        local_mode: false,
        profile: None,
        test_private_key: Some(pri_key_string), // Note: the cli will get a new app_cfg instance and any fields populated at runtime are lost
    };

    cli.run().await.expect("could not run cli");

    let p = next_proof::get_next_proof_params_from_local(&app_cfg)
        .expect("could not find a proof locally");
    assert!(p.next_height == 1, "not the droid");

    // 2. Submit that proof to chain.
    cli.command = TowerSub::Backlog { show: false };
    cli.run().await.expect("could not run cli");

    let (_total_height, submitted_in_epoch) = backlog::get_remote_tower_height(&app_cfg)
        .await
        .expect("could not get remote height");
    assert!(submitted_in_epoch == 1, "chain state not expected");

    // 3. Continue
    // TODO: how to run `tower start` for only a few blocks?
    cli.command = TowerSub::Once;
    cli.run().await.expect("could not run cli");
    cli.command = TowerSub::Backlog { show: false };
    cli.run().await.expect("could not run cli");
    let (_total_height, submitted_in_epoch) = backlog::get_remote_tower_height(&app_cfg)
        .await
        .expect("could not get remote height");
    assert!(submitted_in_epoch == 2, "chain state not expected");

    // 4. Remove block files, and resume from chain state
    let block_dir = app_cfg.get_block_dir(None).unwrap();
    std::fs::remove_dir_all(&block_dir).unwrap();

    cli.command = TowerSub::Once;
    cli.run().await.expect("could not run cli");
    cli.command = TowerSub::Backlog { show: false };
    cli.run().await.expect("could not run cli");
    let (_total_height, submitted_in_epoch) = backlog::get_remote_tower_height(&app_cfg)
        .await
        .expect("could not get remote height");
    assert!(submitted_in_epoch == 3, "chain state not expected");
}
