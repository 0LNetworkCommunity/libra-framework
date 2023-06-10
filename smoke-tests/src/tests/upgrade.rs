use std::{path::PathBuf, process::Command};

use zapatos_smoke_test::smoke_test_environment::{
  new_local_swarm_with_release,
};
use libra_framework::release::ReleaseTarget;
use zapatos_forge::{Node, Swarm};
use zapatos_sdk::types::LocalAccount;
// use zapatos_crypto::traits::ValidCryptoMaterialStringExt;
use std::process::Stdio;
use libra_cached_packages::aptos_stdlib::aptos_governance_create_proposal_v2;
use zapatos::test::CliTestFramework;

#[tokio::test]
async fn test_upgrade_flow() {

    let release = ReleaseTarget::Head.load_bundle().unwrap();
    // let (mut swarm, mut cli, _) = swarm_builder_with_release(4, release).build_with_cli(1).await;

    let mut swarm = new_local_swarm_with_release(4, release).await;
    // let public_info = swarm.aptos_public_info();
    let endpoint = swarm.validators_mut().next().unwrap().rest_api_endpoint();
    let tool = CliTestFramework::new(
        endpoint.clone(),
        endpoint,
        0,
    );

    // let c = public_info.client();

    // let url = public_info.url().to_string();
    // let private_key = public_info
    //     .root_account()
    //     .private_key()
    //     .to_encoded_string()
    //     .unwrap();

    // let proposal_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //     .join("src").join("tests").join("fixtures").join("proposal_script").join("script.mv");
    // dbg!(&proposal_path);
    // assert!(&proposal_path.exists());

    // let v = swarm.validators_mut().next().unwrap();
    // let pri_key = v.account_private_key().as_ref().unwrap();
    // // pri_key.private_key()
    // let address = v.peer_id().to_owned();
    // let mut account = LocalAccount::new(v.peer_id(), pri_key.private_key(), 0);
    

    // let payload = aptos_governance_create_proposal_v2(
    //     // self.pool_address_args.pool_address,
    //     // account.
    //     v.peer_id(),
    //     vec![],
    //     vec![],
    //     vec![],
    //     true,
    // );

    // let mut public_info: zapatos_forge::AptosPublicInfo = swarm.aptos_public_info();


    // let balance = public_info.client()
    //     .get_account_balance(address)
    //     .await
    //     .unwrap()
    //     .into_inner();

    // dbg!(&balance);


    // public_info.mint(address, 10_000_000).await.unwrap();

    // let balance = public_info.client()
    //     .get_account_balance(address)
    //     .await
    //     .unwrap()
    //     .into_inner();

    // dbg!(&balance);

    // let txn = account.sign_with_transaction_builder(
    //     public_info.transaction_factory()
    //         .payload(payload),
    // );

    // let res = public_info.client().submit_and_wait(&txn).await.unwrap();
    // dbg!(&res);

    // check the network still runs
    // check_create_mint_transfer(&mut env).await;
}

#[test]
fn test_command() {
  let cmd = Command::new("vendor")
    // .current_dir(&proposal_path)
    .args(&vec![
        "move",
        "run-script",
        "--assume-yes",
    ])
    .stdout(Stdio::inherit())
    .output()
    .unwrap();

    assert!(cmd.status.success());

}