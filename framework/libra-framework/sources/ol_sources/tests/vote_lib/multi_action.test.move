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
  use diem_framework::resource_account;
  use diem_framework::reconfiguration;
  use diem_framework::account;

  use diem_std::debug::print;

  struct DummyType has drop, store {}  

  #[test(root = @ol_framework, carol = @0x1000c)]
  fun init_multi_action(root: &signer, carol: &signer) {
    mock::genesis_n_vals(root, 2);

    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    multi_action::init_gov(&resource_sig);
    multi_action::init_type<DummyType>(&resource_sig, true);
  }

  // Happy Day: propose offer to authorities
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  fun propose_offer(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    let carol_address = @0x1000c;
    
    // check the offer does not exist
    assert!(!multi_action::exists_offer(carol_address), 0);
    assert!(!multi_action::is_multi_action(carol_address), 0);

    // initialize the multi_action account
    multi_action::init_gov(carol);

    // offer authorities
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::some(3));

    // check the offer is proposed and account is not muti_action yet
    assert!(multi_action::exists_offer(carol_address), 0);
    assert!(multi_action::get_offer_proposed(carol_address) == authorities, 0);
    assert!(vector::is_empty(&multi_action::get_offer_claimed(carol_address)), 0);
    assert!(multi_action::get_offer_expiration_epoch(carol_address) == 3, 0);
    assert!(!multi_action::is_multi_action(carol_address), 0);
  }

  // Happy Day: claim offer by authorities
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  fun claim_offer(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 2);
    let carol_address = @0x1000c;

    // initialize the multi_action account
    multi_action::init_gov(carol);

    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::none());

    // bob claim the offer
    multi_action::claim_offer(bob, carol_address);

    // check the claimed offer
    assert!(multi_action::exists_offer(carol_address), 0);
    let claimed = vector::singleton(signer::address_of(bob));
    let proposed = vector::singleton(signer::address_of(alice));
    assert!(multi_action::get_offer_claimed(carol_address) == claimed, 0);
    assert!(multi_action::get_offer_proposed(carol_address) == proposed, 0);

    // alice claim the offer
    multi_action::claim_offer(alice, carol_address);

    // check alice and bob claimed the offer
    let claimed = multi_action::get_offer_claimed(carol_address);
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(bob));
    vector::push_back(&mut authorities, signer::address_of(alice));
    assert!(claimed == authorities, 0);
    assert!(multi_action::get_offer_proposed(carol_address) == vector::empty(), 0);
  }

  // Happy Day: finalize multisign account
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  fun finalize_multi_action(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    let carol_address = @0x1000c;

    // initialize the multi_action account
    multi_action::init_gov(carol);

    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::none());

    // authorities claim the offer
    multi_action::claim_offer(alice, carol_address);
    multi_action::claim_offer(bob, carol_address);

    // finalize the multi_action account
    assert!(account::exists_at(carol_address), 666);
    multi_action::finalize_and_cage2(carol);

    // check the account is multi_action
    assert!(multi_action::is_multi_action(carol_address), 0);
    
    // check authorities
    let authorities = multi_action::get_authorities(carol_address);
    let claimed = vector::empty<address>();
    vector::push_back(&mut claimed, signer::address_of(alice));
    vector::push_back(&mut claimed, signer::address_of(bob));
    print(&authorities);
    assert!(authorities == claimed, 0);

    // check offer was removed
    assert!(!multi_action::exists_offer(carol_address), 0);
  }

  // Try to claim expired offer
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 15, location = ol_framework::multi_action)]
  fun claim_expired_offer(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 2);
    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);

    // initialize the multi_action account
    multi_action::init_gov(&resource_sig);

    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(&resource_sig, authorities, option::some(2));

    // alice claim the offer
    multi_action::claim_offer(alice, new_resource_address);
    
    mock::trigger_epoch(root); // epoch 1 valid
    mock::trigger_epoch(root); // epoch 2 valid
    mock::trigger_epoch(root); // epoch 3 expired

    // bob claim expired offer
    multi_action::claim_offer(bob, new_resource_address);
  }

  #[test]
  fun test_adal() {
    let authorities: vector<address> = vector::empty();
    let alice = @0x1000a;
    let bob = @0x1000b;
    vector::push_back(&mut authorities, alice);
    vector::push_back(&mut authorities, bob);
    assert!(vector::length(&authorities) == 2, 0);
    assert!(*vector::borrow(&authorities, 0) == alice, 0);
    print(&authorities);

    let x = 0;
    let y = 1;
    let r = if (false) {
      &mut x
    } else {
      &mut y
    };
    *r = *r + 1;
    assert!(*r == 2, 0);
    print(r);

    let text = x"ADA1";
    print(&text);

    let a = 0;
    let b = copy a + 1;
    let c = a + 2;

    print(&b);
    print(&c);
  }

  /*
  #[test(carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  fun offer_authorities(carol: &signer, alice: &signer, bob: &signer) {
    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    print(&1111111);

    // offer authorities to resource_sig for alice and bob
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    // multi_action::propose_offer(&resource_sig, authorities, 7);

    //print(&resource_sig);

  }*/

  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a)]
  fun propose_action(root: &signer, carol: &signer, alice: &signer) {

    let vals = mock::genesis_n_vals(root, 2);
    // mock::ol_initialize_coin(root);

    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    // SO ALICE IS AUTHORIZED
    multi_action::init_gov(&resource_sig);
    multi_action::init_type<DummyType>(&resource_sig, true);

    //need to be caged to finalize multi action workflow and release control of the account
    multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

    // create a proposal
    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());
    let id = multi_action::propose_new(alice, new_resource_address, proposal);

    // SHOULD NOT HAVE COUNTED ANY VOTES
    let v = multi_action::get_votes<DummyType>(new_resource_address, guid::id_creation_num(&id));
    assert!(vector::length(&v) == 0, 7357003);
  }

  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000a)]
  fun propose_action_prevent_duplicated(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    // Scenario: alice and bob are authorities. They try to send the same proposal
    let vals = mock::genesis_n_vals(root, 2);
    // mock::ol_initialize_coin(root);

    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe
    // SO ALICE IS AUTHORIZED
    multi_action::init_gov(&resource_sig);
    multi_action::init_type<DummyType>(&resource_sig, true);

    //need to be caged to finalize multi action workflow and release control of the account
    multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

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

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun vote_action_happy_simple(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
    // Scenario: a simple MultiAction where we don't need any capabilities. Only need to know if the result was successful on the vote that crossed the threshold.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);

    // make the vals the signers on the safe, and 2-of-2 need to sign
    multi_action::init_gov(&resource_sig);
    // Ths is a simple multi_action: there is no capability being stored
    multi_action::init_type<DummyType>(&resource_sig, false);
    //need to be caged to finalize multi action workflow and release control of the account
    multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

    // create a proposal
    let proposal = multi_action::proposal_constructor(DummyType{}, option::none());

    let id = multi_action::propose_new<DummyType>(alice, new_resource_address, proposal);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(alice, &id, new_resource_address);
    assert!(passed == false, 7357001);
    option::destroy_none(cap_opt);

    let (passed, cap_opt) = multi_action::vote_with_id<DummyType>(bob, &id,
    new_resource_address);

    assert!(passed == true, 7357002);

    // THE WITHDRAW CAPABILITY IS MISSING AS EXPECTED
    assert!(option::is_none(&cap_opt), 7357003);

    option::destroy_none(cap_opt);

  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun vote_action_happy_withdraw_cap(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
    // Scenario: testing that a payment type multisig could be created with this module: that the WithdrawCapability can be used here.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);

    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol,
    b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 0);
    // fund the multi_action's account
    ol_account::transfer(alice, new_resource_address, 100);

    // make the vals the signers on the safe, and 2-of-2 need to sign
    multi_action::init_gov(&resource_sig);
    multi_action::init_type<DummyType>(&resource_sig, true);

    //need to be caged to finalize multi action workflow and release control of the account

    multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

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
    ol_account::create_account(root, @0x1000c);
    ol_account::deposit_coins(@0x1000c, c);
    option::fill(&mut cap_opt, cap);

    let (_, balance) = ol_account::balance(@0x1000c);
    assert!(balance == 42, 7357004);


    multi_action::maybe_restore_withdraw_cap(cap_opt);
  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  #[expected_failure(abort_code = 65548, location = ol_framework::multi_action)]

  fun vote_action_expiration(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
    // Scenario: Testing that if an action expires voting cannot be done.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);
    // we are at epoch 0
    let epoch = reconfiguration::get_current_epoch();
    assert!(epoch == 0, 7357001);
    // Dave creates the resource account. He is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 7357002);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and DAVE ARE AUTHORIZED
    safe::init_payment_multisig(&resource_sig); // both need to sign

    //need to be caged to finalize multi action workflow and release control of the account
    multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

    // make a proposal for governance, expires in 2 epoch from now
    let id = multi_action::propose_governance(alice, new_resource_address, vector::empty(), true, option::some(1), option::some(2));

    mock::trigger_epoch(root); // epoch 1
    mock::trigger_epoch(root); // epoch 2
    mock::trigger_epoch(root); // epoch 3 -- voting ended here
    mock::trigger_epoch(root); // epoch 4 -- now expired

    let epoch = reconfiguration::get_current_epoch();

    assert!(epoch == 4, 7357003);

    // trying to vote on a closed ballot will error
    let _passed = multi_action::vote_governance(bob, new_resource_address, &id);

  }


  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, marlon_rando = @0x123456)]
  fun governance_change_auths(root: &signer, alice: &signer, bob: &signer, carol: &signer, marlon_rando: &signer) {
    // Scenario: The multisig gets initiated with the 2 validators as the only authorities. IT takes 2-of-2 to sign.
    // later they add a third (Rando) so it becomes a 2-of-3.
    // Rando and Bob, then remove alice so it becomes 2-of-2 again

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);
    // Dave creates the resource account. HE is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 7357001);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and BOB ARE AUTHORIZED
    multi_action::init_gov(&resource_sig);// both need to sign
    multi_action::init_type<DummyType>(&resource_sig, true);

    //need to be caged to finalize multi action workflow and release control of
    // the account
    multi_action::finalize_and_cage(&resource_sig, vals, 2);

    // Alice is going to propose to change the authorities to add Rando
    let id = multi_action::propose_governance(alice, new_resource_address,
    vector::singleton(signer::address_of(marlon_rando)), true, option::none(),
    option::none());

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

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun governance_change_threshold(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
    // Scenario: The multisig gets initiated with the 2 validators as the only authorities. IT takes 2-of-2 to sign.
    // They decide next only 1-of-2 will be needed.

    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);
    // Dave creates the resource account. HE is not one of the validators, and is not an authority in the multisig.
    let (resource_sig, _cap) = ol_account::test_ol_create_resource_account(carol, b"0x1");
    let new_resource_address = signer::address_of(&resource_sig);
    assert!(resource_account::is_resource_account(new_resource_address), 7357001);

    // fund the account
    ol_account::transfer(alice, new_resource_address, 100);
    // make the vals the signers on the safe
    // SO ALICE and BOB ARE AUTHORIZED
    multi_action::init_gov(&resource_sig);// both need to sign
    multi_action::init_type<DummyType>(&resource_sig, false); // simple type with no capability

    //need to be caged to finalize multi action workflow and release control of the account
    multi_action::finalize_and_cage(&resource_sig, vals, vector::length(&vals));

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
