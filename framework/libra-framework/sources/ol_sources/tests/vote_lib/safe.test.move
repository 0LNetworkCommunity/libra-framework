    // let bal = DiemAccount::balance<GAS>(@DaveMultiSig);
    // assert!(bal == 1000000, 7357001);


    // let addr = Vector::singleton<address>(@Alice);
    // Vector::push_back(&mut addr, @Bob);

    // MultiSig::init_gov(&d_sig, 2, &addr);
    // MultiSig::init_type<PaymentType>(&d_sig, true);
    // MultiSig::finalize_and_brick(&d_sig);
#[test_only]

module ol_framework::test_safe {
  use ol_framework::mock;
  use ol_framework::safe;
  use ol_framework::safe_payment;
  use std::signer;
  use std::option;
  use std::vector;
  use ol_framework::ol_account;
  use aptos_framework::resource_account;
  use ol_framework::gas_coin;
  use ol_framework::slow_wallet;
  use aptos_framework::reconfiguration;

  struct DummyType has drop, store {}

  #[test(root = @ol_framework, dave = @0x1000d)]
  fun init_safe(root: &signer, dave: &signer) {


    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);


    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    safe::init_gov(&resource_sig, 2, &vals);
    safe::init_type<DummyType>(&resource_sig, true);

  }


  #[test(root = @ol_framework, dave = @0x1000d, alice = @0x1000a)]
  fun propose_action(root: &signer, dave: &signer, alice: &signer) {

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);

    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    // SO ALICE IS AUTHORIZED
    safe::init_gov(&resource_sig, 2, &vals);
    safe::init_type<DummyType>(&resource_sig, true);

    let proposal = safe::proposal_constructor(DummyType{}, option::none()); // ends at epoch 10
    safe::propose_new(alice, new_resource_address, proposal);

  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, dave = @0x1000d )]
  fun propose_payment_happy(root: &signer, alice: &signer, bob: &signer, dave: &signer, ) {
    use ol_framework::slow_wallet;

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);
    // Dave creates the resource account. HE is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and DAVE ARE AUTHORIZED
    safe_payment::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // first make sure dave is initialized to receive GasCoin
    ol_account::create_account(root, @0x1000d);
    // when alice proposes, she also votes in a single step
    let prop_id = safe_payment::propose_payment(alice, new_resource_address, @0x1000d, 42, b"cheers", option::none()); // ends at epoch 10

    // vote should pass after bob also votes
    let passed = safe_payment::vote_payment(bob, new_resource_address, &prop_id);
    assert!(passed, 1);
    let (_, total_dave) = slow_wallet::balance(@0x1000d);
    assert!(total_dave == 42, 2);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, eve = @0x1000e)]
  fun propose_payment_should_fail(root: &signer, alice: &signer, bob: &signer, eve: &signer) {
    use ol_framework::slow_wallet;

    let vals = mock::genesis_n_vals(root, 4);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);
    // Eve creates the resource account, is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::ol_create_resource_account(eve, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE, BOB, CAROL, DAVE ARE AUTHORIZED
    // not enough voters
    safe_payment::init_payment_multisig(&resource_sig, vals, 3); // requires 3

    // first make sure EVE is initialized to receive GasCoin
    ol_account::create_account(root, @0x1000e);
    // when alice proposes, she also votes in a single step
    let prop_id = safe_payment::propose_payment(alice, new_resource_address, @0x1000e, 42, b"cheers", option::none()); // ends at epoch 10

    // vote should pass after bob also votes
    let passed = safe_payment::vote_payment(bob, new_resource_address, &prop_id);
    assert!(!passed, 1);
    let (_, total_dave) = slow_wallet::balance(@0x1000e);
    assert!(total_dave == 0, 2);
  }

  #[test(root = @ol_framework, alice = @0x1000a, dave = @0x1000d )]
  fun safe_root_security_fee(root: &signer, alice: &signer, dave: &signer, ) {

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 1000000000000);
    // Dave creates the resource account. HE is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);
    safe_payment::init_payment_multisig(&resource_sig, vals, 3); // requires 3

    // fund the account
    ol_account::transfer(alice, new_resource_address, 1000000);
    let bal = gas_coin::get_balance(new_resource_address);
    mock::trigger_epoch(root);
    let new_bal = gas_coin::get_balance(new_resource_address);
    assert!(new_bal < bal, 735701);
  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, dave = @0x1000d, marlon_rando = @0x2000a)]
  fun governance_change_auths(root: &signer, alice: &signer, bob: &signer, dave: &signer, marlon_rando: &signer) {
    // Scenario: The multisig gets initiated with the 2 validators as the only authorities. IT takes 2-of-2 to sign.
    // later they add a third (Rando) so it becomes a 2-of-3.
    // Rando and Bob, then remove alice so it becomes 2-of-2 again

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);
    // Dave creates the resource account. HE is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 7357001);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and DAVE ARE AUTHORIZED
    safe_payment::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // Alice is going to propose to change the authorities to add Rando

    let id = safe::propose_governance(alice, new_resource_address, vector::singleton(signer::address_of(marlon_rando)), true, option::none(), option::none());
    let a = safe::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357002);

    // bob votes and it becomes final. Bob could either use vote_governance()
    let passed = safe::vote_governance(bob, new_resource_address, &id);
    assert!(passed, 7357003);
    let a = safe::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 3, 7357003);
    assert!(safe::is_authority(new_resource_address, signer::address_of(marlon_rando)), 7357004);

    // Now Rando and Bob, will conspire to remove alice.
    // NOTE: false means remove here
    let id = safe::propose_governance(marlon_rando, new_resource_address, vector::singleton(signer::address_of(alice)), false, option::none(), option::none());
    let a = safe::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 3, 7357002); // no change yet

    // bob votes and it becomes final. Bob could either use vote_governance()
    let passed = safe::vote_governance(bob, new_resource_address, &id);
    assert!(passed, 7357003);
    let a = safe::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357003);
    assert!(!safe::is_authority(new_resource_address, signer::address_of(alice)), 7357004);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, dave = @0x1000d)]
  fun governance_change_threshold(root: &signer, alice: &signer, bob: &signer, dave: &signer) {
    // Scenario: The multisig gets initiated with the 2 validators as the only authorities. IT takes 2-of-2 to sign.
    // They decide next only 1-of-2 will be needed.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);
    // Dave creates the resource account. HE is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 7357001);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and DAVE ARE AUTHORIZED
    safe_payment::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // Alice is going to propose to change the authorities to add Rando

    let id = safe::propose_governance(alice, new_resource_address, vector::empty(), true, option::some(1), option::none());

    let a = safe::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357002); // no change
    let (n, _m) = safe::get_threshold(new_resource_address);
    assert!(n == 2, 7357003);


    // bob votes and it becomes final. Bob could either use vote_governance()
    let passed = safe::vote_governance(bob, new_resource_address, &id);
    assert!(passed, 7357004);
    let a = safe::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357005); // no change
    let (n, _m) = safe::get_threshold(new_resource_address);
    assert!(n == 1, 7357006);

    // make sure dave is properly configured to receive GasCoin
    ol_account::create_account(root, @0x1000d);
    let (_, total_dave) = slow_wallet::balance(@0x1000d);
    assert!(total_dave == 0, 7357007);
    // confirm only one is needed to make a transaction
    // proposal will automatically pass since there's only one voter needed.
    let _prop_id = safe_payment::propose_payment(alice, new_resource_address, @0x1000d, 42, b"cheers", option::none()); // ends at epoch 10
    let (_, total_dave) = slow_wallet::balance(@0x1000d);
    assert!(total_dave == 42, 7357008);

  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, dave = @0x1000d)]
  #[expected_failure(abort_code = 505547, location = ol_framework::safe)]

  fun action_expiration(root: &signer, alice: &signer, bob: &signer, dave: &signer) {
    // Scenario: Testing that if an action expires voting cannot be done.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);
    // make this epoch 1
    // mock::trigger_epoch(root);

    // Dave creates the resource account. He is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 7357001);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and DAVE ARE AUTHORIZED
    safe_payment::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // make a proposal for governance, expires in 2 epoch from now
    let id = safe::propose_governance(alice, new_resource_address, vector::empty(), true, option::some(1), option::some(2));

    mock::trigger_epoch(root); // epoch 1
    mock::trigger_epoch(root); // epoch 2
    mock::trigger_epoch(root); // epoch 3 -- voting ended here
    mock::trigger_epoch(root); // epoch 4 -- now expired

    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 4, 7357004);

    // trying to vote on a closed ballot will error
    let _passed = safe::vote_governance(bob, new_resource_address, &id);

  }
}