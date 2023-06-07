use std::{path::PathBuf, process::Command};

use aptos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use aptos_forge::Swarm;

use zapatos_crypto::traits::ValidCryptoMaterialStringExt;

#[tokio::test]
async fn test_upgrade_flow() {
    // // prebuild tools.
    // // let aptos_cli = workspace_builder::get_bin("aptos");

    // let num_nodes = 5;
    // let (mut env, _cli, _) = SwarmBuilder::new_local(num_nodes)
    //     .with_aptos_testnet()
    //     .build_with_cli(0)
    //     .await;

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    let mut swarm = new_local_swarm_with_release(4, release).await;
    let mut public_info = swarm.aptos_public_info();

    let url = public_info.url().to_string();
    let private_key = public_info
        .root_account()
        .private_key()
        .to_encoded_string()
        .unwrap();

    let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests").join("fixtures");

    let framework_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().join("framework").join("libra-framework");

    assert!(Command::new("vendor")
    .current_dir(&proposal_path)
    .args(&vec![
        "move",
        "run-script",
        "--script-path",
        &proposal_path.to_str().unwrap(),
        "--framework-local-dir",
        framework_path.as_os_str().to_str().unwrap(),
        "--sender-account",
        "0xA550C18",
        "--url",
        url.as_str(),
        "--private-key",
        private_key.as_str(),
        "--assume-yes",
    ])
    .output()
    .unwrap()
    .status
    .success());

    // //TODO: Make sure gas schedule is indeed updated by the tool.

    // // Test the module publishing workflow
    // let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    // let base_path_v1 = base_dir.join("src/aptos/package_publish_modules_v1/");

    // move_test_helpers::publish_package(&mut env.aptos_public_info(), base_path_v1)
    //     .await
    //     .unwrap();

    // check_create_mint_transfer(&mut env).await;
}
