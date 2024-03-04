use diem_sdk::crypto::ed25519::Ed25519PrivateKey;
use diem_sdk::crypto::{Uniform, ValidCryptoMaterialStringExt};
use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::submit_transaction::Sender;
use libra_txs::txs_cli_user::{OfferRotationCapabilityTx, RotateKeyTx};
use libra_types::legacy_types::app_cfg::{Profile};
use libra_wallet::account_keys;

// Scenario: We have an initial validator, Val 0 with a random address
// create an account for Alice (with a known address and mnemonic)
// rotate key for Alice's account using cli

/// Test key rotation
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore]
async fn rotate_key() -> anyhow::Result<()> {
    // create libra swarm and get app config for the first validator
    let mut ls = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");
    let mut val_app_cfg = ls.first_account_app_cfg()?;

    // get an appcfg struct from Alice's mnemonic
    let alice = account_keys::get_keys_from_mnem("talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse".to_owned())?;
    let alice_acct = &alice.child_0_owner.account;

    // create an account for alice by transferring funds
    let mut s = Sender::from_app_cfg(&val_app_cfg, None).await?;
    let res = s
        .transfer(alice.child_0_owner.account, 100.0, false)
        .await?
        .unwrap();
    assert!(res.info.status().is_success());
    println!(
        "alice: {:?} auth: {:?} pri: {:?}",
        alice.child_0_owner.account,
        alice.child_0_owner.auth_key.to_string(),
        alice.child_0_owner.pri_key.to_string(),
    );

    let mut p = Profile::new(alice.child_0_owner.auth_key, alice.child_0_owner.account);
    assert!(alice_acct == &p.account);

    p.set_private_key(&alice.child_0_owner.pri_key);

    val_app_cfg.maybe_add_profile(p)?;

    // also checking we can get a Sender type with a second profile
    let mut alice_sender =
        Sender::from_app_cfg(&val_app_cfg, Some(alice.child_0_owner.account.to_string())).await?;

    assert!(alice_acct == &alice_sender.local_account.address());

    let original_auth_key = alice.child_0_owner.auth_key.to_string();
    println!("original_auth_key: {:?}", original_auth_key);

    // check auth key from chain
    let account_query = ls.client().get_account(alice.child_0_owner.account).await;
    assert!(account_query.is_ok());
    let account_from_chain = account_query.unwrap().into_inner();
    let auth_key_from_chain = account_from_chain.authentication_key.to_string();
    assert_eq!(auth_key_from_chain, original_auth_key);

    // generate new private key from which auth key will be generated
    let generated_private_key = Ed25519PrivateKey::generate_for_testing();
    let generated_private_key_encoded =
        Ed25519PrivateKey::to_encoded_string(&generated_private_key);
    assert!(generated_private_key_encoded.is_ok());

    let cli = RotateKeyTx {
        new_private_key: Some(generated_private_key_encoded.unwrap()),
    };

    let res = cli.run(&mut alice_sender).await;
    assert!(res.is_ok());

    // check new auth key
    let updated_account_query = ls.client().get_account(alice.child_0_owner.account).await;
    assert!(updated_account_query.is_ok());
    let account_updated = updated_account_query.unwrap().into_inner();
    let new_auth_key = account_updated.authentication_key.to_string();
    println!("new_auth_key: {:?}", new_auth_key);

    assert_ne!(new_auth_key, original_auth_key);

    Ok(())
}

/// Test rotation capability offer
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn offer_rotation_capability() -> anyhow::Result<()> {
    // create libra swarm and get app config for the first validator
    let mut ls = LibraSmoke::new(Some(1))
        .await
        .expect("could not start libra smoke");
    let mut val_app_cfg = ls.first_account_app_cfg()?;

    // get an appcfg struct from Alice's mnemonic
    let alice = account_keys::get_keys_from_mnem("talent sunset lizard pill fame nuclear spy noodle basket okay critic grow sleep legend hurry pitch blanket clerk impose rough degree sock insane purse".to_owned())?;
    let alice_acct = &alice.child_0_owner.account;

    // create an account for alice by transferring funds
    let mut s = Sender::from_app_cfg(&val_app_cfg, None).await?;
    let res = s
        .transfer(alice.child_0_owner.account, 100.0, false)
        .await?
        .unwrap();
    assert!(res.info.status().is_success());
    println!(
        "alice: {:?} auth: {:?} pri: {:?}",
        alice.child_0_owner.account,
        alice.child_0_owner.auth_key.to_string(),
        alice.child_0_owner.pri_key.to_string(),
    );

    let mut p = Profile::new(alice.child_0_owner.auth_key, alice.child_0_owner.account);
    assert!(alice_acct == &p.account);

    p.set_private_key(&alice.child_0_owner.pri_key);

    val_app_cfg.maybe_add_profile(p)?;

    // also checking we can get a Sender type with a second profile
    let mut alice_sender =
        Sender::from_app_cfg(&val_app_cfg, Some(alice.child_0_owner.account.to_string())).await?;

    assert!(alice_acct == &alice_sender.local_account.address());

    let original_auth_key = alice.child_0_owner.auth_key.to_string();
    println!("original_auth_key: {:?}", original_auth_key);

    // create a new account by transferring fundsF
    let bob_account = ls.marlon_rando();
    let res_bob = s
        .transfer(bob_account.address(), 100.0, false)
        .await?
        .unwrap();
    assert!(res_bob.info.status().is_success());

    let cli = OfferRotationCapabilityTx {
        delegate_address: bob_account.address().to_string(),
    };

    let res = cli.run(&mut alice_sender).await;
    match res.as_ref() {
        Ok(_) => {}
        Err(err) => {println!("got error={:?}", err)}
    }
    assert!(res.is_ok());

    Ok(())
}
