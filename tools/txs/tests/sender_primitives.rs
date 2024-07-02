use libra_smoke_tests::libra_smoke::LibraSmoke;
use libra_txs::submit_transaction::Sender;
use libra_types::core_types::app_cfg::Profile;
use libra_wallet::account_keys;

// Scenario: We have an initial validator, Val 0 with a random address
// create an account for Alice (with a known address and mnemonic)
// we will use sender.transfer to send money from Val 0 to Alice
// then back from Alice to Val 0
// We are also checking that we can have multiple profiles on AppCfg
// and that we can choose between them to send transactions.

/// Testing that we can get a swarm up with the current head.mrb
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn sender_back_and_forth() -> anyhow::Result<()> {
    // create libra swarm and get app config for the first validator
    let mut ls = LibraSmoke::new(Some(1), None)
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

    let mut p = Profile::new(alice.child_0_owner.auth_key, alice.child_0_owner.account);
    assert!(alice_acct == &p.account);

    p.set_private_key(&alice.child_0_owner.pri_key);

    val_app_cfg.maybe_add_profile(p)?;

    // also checking we can get a Sender type with a second profile
    let mut alice_sender =
        Sender::from_app_cfg(&val_app_cfg, Some(alice.child_0_owner.account.to_string())).await?;

    assert!(alice_acct == &alice_sender.local_account.address());

    let res = alice_sender
        .transfer(ls.first_account.address(), 10.0, false)
        .await?
        .unwrap();

    assert!(res.info.status().is_success());

    Ok(())
}
