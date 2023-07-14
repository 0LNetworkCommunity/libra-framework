
#[test_only]
module ol_framework::test_multi_action {
  use ol_framework::mock;
  use ol_framework::multi_action;
  use ol_framework::safe;
  use std::signer;
  use std::option;
  use std::vector;
  use std::guid;
  use ol_framework::ol_account;
  use aptos_framework::resource_account;
  // use ol_framework::slow_wallet;
  use aptos_framework::reconfiguration;
  use aptos_framework::coin;

  // use aptos_std::debug::print;

  struct DummyType has drop, store {}

  #[test(root = @ol_framework, dave = @0x1000d)]
  fun init_multi_action(root: &signer, dave: &signer) {


    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);


    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    multi_action::init_gov(&resource_sig, 2, &vals);
    multi_action::init_type<DummyType>(&resource_sig, true);

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

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000a, dave = @0x1000d)]
  fun vote_action_happy(root: &signer, alice: &signer, bob: &signer, dave: &signer) {
    // Scenario: a simple MultiAction where we don't need any capabilities. Only need to know if the result was successful on the vote that crossed the threshold.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);

    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe, and 2-of-2 need to sign
    multi_action::init_gov(&resource_sig, 2, &vals);
    // Ths is a simple multi_action: there is no capability being stored
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


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000a, dave = @0x1000d)]
  fun vote_action_happy_withdraw_cap(root: &signer, alice: &signer, bob: &signer, dave: &signer) {
    // Scenario: testing that a payment type multisig could be created with this module: that the WithdrawCapability can be used here.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000);

    let (resource_sig, _cap) = ol_account::ol_create_resource_account(dave, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);
    // fund the multi_action's account
    ol_account::transfer(alice, new_resource_address, 100);

    // make the vals the signers on the safe, and 2-of-2 need to sign
    multi_action::init_gov(&resource_sig, 2, &vals);
    multi_action::init_type<DummyType>(&resource_sig, true);

    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());

    let id = multi_action::propose_new<DummyType>(alice, new_resource_address, proposal);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(alice, &id, new_resource_address);
    assert!(passed == false, 7357001);
    option::destroy_none(cap_opt);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(bob, &id, new_resource_address);
    assert!(passed == true, 7357002);

    // THE WITHDRAW CAPABILITY IS WHERE WE EXPECT
    assert!(option::is_some(&cap_opt), 7357003);

    let cap = option::extract(&mut cap_opt);
    let c = ol_account::withdraw_with_capability(
      &cap,
      42,
    );
    ol_account::create_account(root, @0x1000d);
    coin::deposit(@0x1000d, c);
    option::fill(&mut cap_opt, cap);

    let (_, balance) = ol_account::balance(@0x1000d);
    assert!(balance == 42, 7357004);


    multi_action::maybe_restore_withdraw_cap(cap_opt);
  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, dave = @0x1000d)]
  #[expected_failure(abort_code = 505547, location = ol_framework::multi_action)]

  fun vote_action_expiration(root: &signer, alice: &signer, bob: &signer, dave: &signer) {
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
    // SO ALICE and BOB ARE AUTHORIZED
    multi_action::init_gov(&resource_sig, 2, &vals);// both need to sign
    multi_action::init_type<DummyType>(&resource_sig, true);

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
    // SO ALICE and BOB ARE AUTHORIZED
    multi_action::init_gov(&resource_sig, 2, &vals);// both need to sign
    multi_action::init_type<DummyType>(&resource_sig, false); // simple type with no capability

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

    // now any other type of action can be taken with just one signer

    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());

    let id = multi_action::propose_new<DummyType>(bob, new_resource_address, proposal);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(bob, &id, new_resource_address);
    assert!(passed == true, 7357002);

    // THE WITHDRAW CAPABILITY IS MISSING AS EXPECTED
    assert!(option::is_none(&cap_opt), 7357003);

    option::destroy_none(cap_opt);
  }
}