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
  use ol_framework::multi_action;
  use ol_framework::safe;
  use std::signer;
  use std::option;
  use std::vector;
  use std::guid;
  use ol_framework::ol_account;
  use aptos_framework::resource_account;
  use ol_framework::gas_coin;
  use ol_framework::slow_wallet;
  use aptos_framework::reconfiguration;

  // use aptos_std::debug::print;

  struct DummyType has drop, store {}

  #[test(root = @ol_framework, dave = @0x1000d)]
  fun init_safe(root: &signer, dave: &signer) {


    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);


    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    multi_action::init_gov(&resource_sig, 2, &vals);
    multi_action::init_type<DummyType>(&resource_sig, true);

  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000a, dave = @0x1000d)]
  fun init_safe_without_withdraw_cap(root: &signer, alice: &signer, bob: &signer, dave: &signer) {
    // Scenario: here we are going to create a frankenstein multisig, and it will not work.
    // we will try to create an action that requires the WithdrawCapability, but on initialization forget to save that capability.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);

    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe, and 2-of-2 need to sign
    multi_action::init_gov(&resource_sig, 2, &vals);
    // WE messed up the initialization here. We created a group action of PaymentType, but we initialized the action with `false` which did not store the WithdrawCapability.
    // this set up will not catch the error, and only when payments are attempted will there be aborted transactions.
    // NOTE: safe_payment includes the proper initialization abstraction
    multi_action::init_type<DummyType>(&resource_sig, false);

    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());

    let id = multi_action::propose_new<DummyType>(alice, new_resource_address, proposal);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(alice, &id, new_resource_address);
    assert!(passed == false, 7357001);
    option::destroy_none(cap_opt);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(bob, &id, new_resource_address);
    assert!(passed == true, 7357002);

    // THE WITHDRAW CAPABILITY IS MISSING AS EXPECTED
    assert!(option::is_none(&cap_opt), 7357003);

    option::destroy_none(cap_opt);

  }


    //   MultiSig::init_gov(&d_sig, 2, &addr);

    // // NOTE: there is no withdraw capability being sent.
    // // so transactions will fail.
    // let withdraw = false;
    // MultiSig::init_type<MultiSigPayment::PaymentType>(&d_sig, withdraw);
    // MultiSig::finalize_and_brick(&d_sig);


  #[test(root = @ol_framework, dave = @0x1000d, alice = @0x1000a)]
  fun propose_action(root: &signer, dave: &signer, alice: &signer) {

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);

    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    // SO ALICE IS AUTHORIZED
    multi_action::init_gov(&resource_sig, 2, &vals);
    multi_action::init_type<DummyType>(&resource_sig, true);

    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());
    let id = multi_action::propose_new(alice, new_resource_address, proposal);

    // SHOULD NOT HAVE COUNTED ANY VOTES
    let v = multi_action::get_votes<DummyType>(new_resource_address, guid::id_creation_num(&id));
    assert!(vector::length(&v) == 0, 7357003);

  }

  #[test(root = @ol_framework, dave = @0x1000d, alice = @0x1000a, bob = @0x1000a)]
  fun propose_action_prevent_duplicated(root: &signer, dave: &signer, alice: &signer, bob: &signer) {
    // Scenario: alice and bob are authorities. They try to send the same proposal
    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);

    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    // SO ALICE IS AUTHORIZED
    multi_action::init_gov(&resource_sig, 2, &vals);
    multi_action::init_type<DummyType>(&resource_sig, true);

    let count = multi_action::get_count_of_pending<DummyType>(new_resource_address);
    assert!(count == 0, 7357001);

    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());
    let id = multi_action::propose_new(alice, new_resource_address, proposal);
    let count = multi_action::get_count_of_pending<DummyType>(new_resource_address);
    assert!(count == 1, 7357002);

    let epoch_ending = multi_action::get_expiration<DummyType>(new_resource_address, guid::id_creation_num(&id));
    assert!(epoch_ending == 14, 7357003);

    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());
    multi_action::propose_new(bob, new_resource_address, proposal);
    let count = multi_action::get_count_of_pending<DummyType>(new_resource_address);
    assert!(count == 1, 7357005); // no change

    // confirm there are no votes
    let v = multi_action::get_votes<DummyType>(new_resource_address, guid::id_creation_num(&id));
    assert!(vector::length(&v) == 0, 7357003);

    // proposing a different ending epoch will have no effect on the proposal once it is started. Here we try to set to epoch 4, but nothing should change, at it will still be 14.
    let proposal = multi_action::proposal_constructor(DummyType{}, option::some(4));
    multi_action::propose_new(bob, new_resource_address, proposal);
    let count = multi_action::get_count_of_pending<DummyType>(new_resource_address);
    assert!(count == 1, 7357004);
    let epoch = multi_action::get_expiration<DummyType>(new_resource_address, guid::id_creation_num(&id));
    assert!(epoch != 4, 7357004);
    assert!(epoch == 14, 7357005);

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
    safe::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // first make sure dave is initialized to receive GasCoin
    ol_account::create_account(root, @0x1000d);
    // when alice proposes, she also votes in a single step
    let prop_id = safe::propose_payment(alice, new_resource_address, @0x1000d, 42, b"cheers", option::none());

    // vote should pass after bob also votes
    let passed = safe::vote_payment(bob, new_resource_address, &prop_id);
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
    safe::init_payment_multisig(&resource_sig, vals, 3); // requires 3

    // first make sure EVE is initialized to receive GasCoin
    ol_account::create_account(root, @0x1000e);
    // when alice proposes, she also votes in a single step
    let prop_id = safe::propose_payment(alice, new_resource_address, @0x1000e, 42, b"cheers", option::none());

    // vote should pass after bob also votes
    let passed = safe::vote_payment(bob, new_resource_address, &prop_id);
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
    safe::init_payment_multisig(&resource_sig, vals, 3); // requires 3

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
    safe::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // Alice is going to propose to change the authorities to add Rando

    let id = multi_action::propose_governance(alice, new_resource_address, vector::singleton(signer::address_of(marlon_rando)), true, option::none(), option::none());
    let a = multi_action::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357002);

    // bob votes and it becomes final. Bob could either use vote_governance()
    let passed = multi_action::vote_governance(bob, new_resource_address, &id);
    assert!(passed, 7357003);
    let a = multi_action::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 3, 7357003);
    assert!(multi_action::is_authority(new_resource_address, signer::address_of(marlon_rando)), 7357004);

    // Now Rando and Bob, will conspire to remove alice.
    // NOTE: false means remove here
    let id = multi_action::propose_governance(marlon_rando, new_resource_address, vector::singleton(signer::address_of(alice)), false, option::none(), option::none());
    let a = multi_action::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 3, 7357002); // no change yet

    // bob votes and it becomes final. Bob could either use vote_governance()
    let passed = multi_action::vote_governance(bob, new_resource_address, &id);
    assert!(passed, 7357003);
    let a = multi_action::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357003);
    assert!(!multi_action::is_authority(new_resource_address, signer::address_of(alice)), 7357004);
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
    safe::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // Alice is going to propose to change the authorities to add Rando

    let id = multi_action::propose_governance(alice, new_resource_address, vector::empty(), true, option::some(1), option::none());

    let a = multi_action::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357002); // no change
    let (n, _m) = multi_action::get_threshold(new_resource_address);
    assert!(n == 2, 7357003);


    // bob votes and it becomes final. Bob could either use vote_governance()
    let passed = multi_action::vote_governance(bob, new_resource_address, &id);
    assert!(passed, 7357004);
    let a = multi_action::get_authorities(new_resource_address);
    assert!(vector::length(&a) == 2, 7357005); // no change
    let (n, _m) = multi_action::get_threshold(new_resource_address);
    assert!(n == 1, 7357006);

    // make sure dave is properly configured to receive GasCoin
    ol_account::create_account(root, @0x1000d);
    let (_, total_dave) = slow_wallet::balance(@0x1000d);
    assert!(total_dave == 0, 7357007);
    // confirm only one is needed to make a transaction
    // proposal will automatically pass since there's only one voter needed.
    let _prop_id = safe::propose_payment(alice, new_resource_address, @0x1000d, 42, b"cheers", option::none());
    let (_, total_dave) = slow_wallet::balance(@0x1000d);
    assert!(total_dave == 42, 7357008);

  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, dave = @0x1000d)]
  #[expected_failure(abort_code = 505547, location = ol_framework::multi_action)]

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
    safe::init_payment_multisig(&resource_sig, vals, 2); // both need to sign

    // make a proposal for governance, expires in 2 epoch from now
    let id = multi_action::propose_governance(alice, new_resource_address, vector::empty(), true, option::some(1), option::some(2));

    mock::trigger_epoch(root); // epoch 1
    mock::trigger_epoch(root); // epoch 2
    mock::trigger_epoch(root); // epoch 3 -- voting ended here
    mock::trigger_epoch(root); // epoch 4 -- now expired

    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 4, 7357004);

    // trying to vote on a closed ballot will error
    let _passed = multi_action::vote_governance(bob, new_resource_address, &id);

  }
}