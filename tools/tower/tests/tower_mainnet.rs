use diem_temppath::TempPath;
use libra_smoke_tests::{configure_validator, libra_smoke::LibraSmoke};

use libra_tower::{
    core::{backlog, next_proof},
    tower_cli::{TowerCli, TowerSub},
};

use libra_txs::submit_transaction::Sender;
use libra_types::{exports::ValidCryptoMaterialStringExt, legacy_types::{block::VDFProof, app_cfg::{AppCfg, Profile}}};
use libra_wallet::account_keys;

// Scenario: We want to start from a blank slate and with the CLI tool:
// 1. have the validator reate a zeroth proof locally
// 2. Submit that proof to chain.
// 3. Continue mining from the previous proof.
// 4. Successfully resume, after nuking all local proofs.
// 5. Continue mining after a new epoch has started. // TODO

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tower_mainnet_difficulty() -> anyhow::Result<()>{
    // create libra swarm and get app config for the first validator
    let mut ls = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");
    let mut val_app_cfg = ls.first_account_app_cfg()?;


    // get an appcfg struct from Alice's mnemonic
    let alice = account_keys::get_keys_from_mnem("talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse".to_owned())?;

    // create an account for alice by transferring funds
    let mut s = Sender::from_app_cfg(&val_app_cfg, None).await?;
    let tx = s.transfer(alice.child_0_owner.account, 10_000.0, false).await?.unwrap();
    dbg!(&tx);

  //   // alice submits the genesis fixture proof with mainnet difficulty
  //  let proof: VDFProof = serde_json::from_str(r#"{
  //     "height": 0,
  //     "elapsed_secs": 42552,
  //     "preimage": "87515d94a244235a1433d7117bc0cb154c613c2f4b1e67ca8d98a542ee3f59f50000000000000000006d61696e6e6574005ed0b2000000005e01000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000050726f74657374732072616765206163726f737320746865206e6174696f6e",
  //     "proof": "14d92a82887d66b226b63f7f228638c3c3695f6786b5066e8a0f25df1bf4cedbcd537701a2218aa7f89b1b67119da2470b5ffe5be0edfca41c7e17519ca10ab4566dff42dfa3918dde618ab8e1b2b14c2f7afe1e067a34d3",
  //     "difficulty": 3000000000,
  //     "security": 350
  //   }"#)?;


  //   // let d = TempPath::new().create_as_dir();
  //   // let's reuse the validators AppCfg since it has all the connection info we want.

  //   let p = Profile::new(alice.child_0_owner.auth_key, alice.child_0_owner.account);

  //   val_app_cfg.maybe_add_profile(p)?;

  //   let mut alice_sender = Sender::from_app_cfg(&mut val_app_cfg, Some(alice.child_0_owner.account.to_hex_literal())).await?;

  //   let res = alice_sender.commit_proof(proof).await?;


    Ok(())
    // let (_, app_cfg) =
    //     configure_validator::init_val_config_files(&mut ls.swarm, 0, d.path().to_owned())
    //         .await
    //         .expect("could not init validator config");

    // // check the tower state is blank
    // assert!(backlog::get_remote_tower_height(&app_cfg).await.is_err());

    // // 1. have the validator reate a zeroth proof locally
    // let profile = app_cfg.get_profile(None).unwrap();
    // let pri_key_string = profile
    //     .borrow_private_key()
    //     .unwrap()
    //     .to_encoded_string()
    //     .unwrap();
    // let mut cli = TowerCli {
    //     command: TowerSub::Zero,
    //     config_file: Some(d.path().join("libra.yaml")),
    //     local_mode: false,
    //     profile: None,
    //     test_private_key: Some(pri_key_string), // Note: the cli will get a new app_cfg instance and any fields populated at runtime are lost
    // };

    // cli.run().await.expect("could not run cli");

    // let p = next_proof::get_next_proof_params_from_local(&app_cfg)
    //     .expect("could not find a proof locally");
    // assert!(p.next_height == 1, "not the droid");

    // // 2. Submit that proof to chain.
    // cli.command = TowerSub::Backlog { show: false };
    // cli.run().await.expect("could not run cli");

    // let (_total_height, submitted_in_epoch) = backlog::get_remote_tower_height(&app_cfg)
    //     .await
    //     .expect("could not get remote height");
    // assert!(submitted_in_epoch == 1, "chain state not expected");

    // // 3. Continue
    // // TODO: how to run `tower start` for only a few blocks?
    // cli.command = TowerSub::Once;
    // cli.run().await.expect("could not run cli");
    // cli.command = TowerSub::Backlog { show: false };
    // cli.run().await.expect("could not run cli");
    // let (_total_height, submitted_in_epoch) = backlog::get_remote_tower_height(&app_cfg)
    //     .await
    //     .expect("could not get remote height");
    // assert!(submitted_in_epoch == 2, "chain state not expected");

    // // 4. Remove block files, and resume from chain state
    // let block_dir = app_cfg.get_block_dir(None).unwrap();
    // std::fs::remove_dir_all(&block_dir).unwrap();

    // cli.command = TowerSub::Once;
    // cli.run().await.expect("could not run cli");
    // cli.command = TowerSub::Backlog { show: false };
    // cli.run().await.expect("could not run cli");
    // let (_total_height, submitted_in_epoch) = backlog::get_remote_tower_height(&app_cfg)
    //     .await
    //     .expect("could not get remote height");
    // assert!(submitted_in_epoch == 3, "chain state not expected");
}
