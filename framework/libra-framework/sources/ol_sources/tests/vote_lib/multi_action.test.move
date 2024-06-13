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
    assert!(multi_action::get_offer_claimed(carol_address) == vector::empty(), 0);
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

  // Propose another offer with different authorities
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
  fun propose_another_offer_different_authorities(root: &signer, alice: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    multi_action::init_gov(alice);

    // invite bob
    multi_action::propose_offer(alice, vector::singleton(@0x1000b), option::some(1));

    // invite carol and dave
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, @0x1000c);
    vector::push_back(&mut authorities, @0x1000d);
    multi_action::propose_offer(alice, authorities, option::some(2));

    // check new authorities
    assert!(multi_action::get_offer_expiration_epoch(@0x1000a) == 2, 0);
    print(&multi_action::get_offer_proposed(@0x1000a));
    assert!(multi_action::get_offer_proposed(@0x1000a) == authorities, 0);
    assert!(multi_action::get_offer_claimed(@0x1000a) == vector::empty(), 0);
  }

  // Propose new offer with more authorities
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
  fun propose_offer_more_authorities(root: &signer, alice: &signer, carol: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    multi_action::init_gov(alice);

    // invite bob
    multi_action::propose_offer(alice, vector::singleton(@0x1000b), option::some(1));

    // new invite bob and carol
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, @0x1000b);
    vector::push_back(&mut authorities, @0x1000c);
    multi_action::propose_offer(alice, authorities, option::some(2));

    // check new authorities
    assert!(multi_action::get_offer_proposed(@0x1000a) == authorities, 0);
    assert!(multi_action::get_offer_claimed(@0x1000a) == vector::empty(), 0);

    // carol claim the offer
    multi_action::claim_offer(carol, @0x1000a);

    // new invite bob, carol and dave
    vector::push_back(&mut authorities, @0x1000d);
    multi_action::propose_offer(alice, authorities, option::some(3));

    // check new authorities
    let proposed = vector::empty();
    vector::push_back(&mut proposed, @0x1000b);
    vector::push_back(&mut proposed, @0x1000d);
    assert!(multi_action::get_offer_expiration_epoch(@0x1000a) == 3, 0);
    assert!(multi_action::get_offer_proposed(@0x1000a) == proposed, 0);
    assert!(multi_action::get_offer_claimed(@0x1000a) == vector::singleton(@0x1000c), 0);
  }

  // Propose new offer with less authorities
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
  fun propose_offer_less_authorities(root: &signer, alice: &signer, carol: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    multi_action::init_gov(alice);

    // invite bob, carol and dave
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, @0x1000b);
    vector::push_back(&mut authorities, @0x1000c);
    vector::push_back(&mut authorities, @0x1000d);
    multi_action::propose_offer(alice, authorities, option::some(2));

    // new invite bob and carol
    let new_authorities = vector::empty<address>();
    vector::push_back(&mut new_authorities, @0x1000b);
    vector::push_back(&mut new_authorities, @0x1000c);
    multi_action::propose_offer(alice, new_authorities, option::some(3));

    // check new authorities minus dave
    assert!(multi_action::get_offer_proposed(@0x1000a) == new_authorities, 0);
    assert!(multi_action::get_offer_claimed(@0x1000a) == vector::empty(), 0);

    // carol claim the offer
    multi_action::claim_offer(carol, @0x1000a);

    // new invite carol
    multi_action::propose_offer(alice, vector::singleton(@0x1000c), option::some(4));

    // check new authorities minus bob
    assert!(multi_action::get_offer_proposed(@0x1000a) == vector::empty(), 0);
    assert!(multi_action::get_offer_claimed(@0x1000a) == vector::singleton(@0x1000c), 0);
  }

  // Propose new offer with same authorities
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun propose_offer_same_authorities(root: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    multi_action::init_gov(alice);

    // invite bob and carol
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, @0x1000b);
    vector::push_back(&mut authorities, @0x1000c);
    multi_action::propose_offer(alice, authorities, option::some(2));

    // new invite bob and carol
    multi_action::propose_offer(alice, authorities, option::some(3));

    // check authorities
    assert!(multi_action::get_offer_proposed(@0x1000a) == authorities, 0);

    // bob claim the offer
    multi_action::claim_offer(bob, @0x1000a);

    // new invite bob and carol
    multi_action::propose_offer(alice, authorities, option::some(4));

    // check authorities
    assert!(multi_action::get_offer_proposed(@0x1000a) == vector::singleton(@0x1000c), 0);
    assert!(multi_action::get_offer_claimed(@0x1000a) == vector::singleton(@0x1000b), 0);
  }

  // Try to propose offer without governance
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x30001, location = ol_framework::multi_action)]
  fun propose_offer_without_gov(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);

    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::none());
  }

  // Try to propose offer to an multisig account
  #[test(root = @ol_framework, dave = @0x1000d, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x30019, location = ol_framework::multi_action)]
  fun propose_offer_to_multisign(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    let carol_address = @0x1000c;
    let dave_address = @0x1000d;
    multi_action::init_gov(carol);
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::none());
    multi_action::claim_offer(alice, carol_address);
    multi_action::claim_offer(bob, carol_address);
    multi_action::finalize_and_cage2(carol);

    // propose offer to multisig account
    multi_action::propose_offer(carol, vector::singleton(dave_address), option::none());
  }

  // Try to propose an empty offer
  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x10016, location = ol_framework::multi_action)]
  fun propose_empty_offer(root: &signer, alice: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    multi_action::init_gov(alice);
    multi_action::propose_offer(alice, vector::empty<address>(), option::none());
  }

  // Try to propose offer to the signer address
  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x50002, location = ol_framework::multi_action)]
  fun propose_offer_to_signer(root: &signer, alice: &signer) {
    let _vals = mock::genesis_n_vals(root, 4);
    let alice_address = signer::address_of(alice);
    multi_action::init_gov(alice);
    multi_action::propose_offer(alice, vector::singleton<address>(alice_address), option::none());
  }

  // Try to propose offer to an invalid signer
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x60021, location = ol_framework::multi_action)]
  fun offer_to_invalid_authority(root: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    multi_action::init_gov(alice);

    // propose to invalid address
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(bob));
    vector::push_back(&mut authorities, @0xCAFE);
    multi_action::propose_offer(alice, authorities, option::some(2));
  }

  // Try to propose offer with zero duration epochs
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x10022, location = ol_framework::multi_action)]
  fun offer_with_zero_duration(root: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    multi_action::init_gov(alice);

    // propose to invalid address
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(alice, authorities, option::some(0));
  }

  // Try to claim offer not offered to signer
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  #[expected_failure(abort_code = 0x60020, location = ol_framework::multi_action)]
  fun claim_offer_not_offered(root: &signer, alice: &signer, carol: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    let carol_address = @0x1000c;
    multi_action::init_gov(carol);
    
    // invite bob
    multi_action::propose_offer(carol, vector::singleton(@0x1000b), option::none());

    // alice try to claim the offer
    multi_action::claim_offer(alice, carol_address);
  }

  // Try to claim expired offer
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x20015, location = ol_framework::multi_action)]
  fun claim_expired_offer(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    let carol_address = @0x1000c;
    multi_action::init_gov(carol);
    
    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::some(2));

    // alice claim the offer
    multi_action::claim_offer(alice, carol_address);
    
    mock::trigger_epoch(root); // epoch 1 valid
    mock::trigger_epoch(root); // epoch 2 valid
    mock::trigger_epoch(root); // epoch 3 expired

    // bob claim expired offer
    multi_action::claim_offer(bob, carol_address);
  }

  // Try to claim offer of an account without proposal
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]  
  #[expected_failure(abort_code = 0x60017, location = ol_framework::multi_action)]
  fun claim_offer_without_proposal(root: &signer, alice: &signer) {
    let _vals = mock::genesis_n_vals(root, 2);
    let bob_address = @0x1000c;
    multi_action::claim_offer(alice, bob_address);
  }

  // Try to claim offer twice
  #[test(root = @ol_framework, carol = @0x1000c, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x80023, location = ol_framework::multi_action)]
  fun claim_offer_twice(root: &signer, carol: &signer, alice: &signer, bob: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    let carol_address = @0x1000c;
    multi_action::init_gov(carol);
    
    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(alice));
    vector::push_back(&mut authorities, signer::address_of(bob));
    multi_action::propose_offer(carol, authorities, option::some(2));

    // Alice claim the offer twice
    multi_action::claim_offer(alice, carol_address);
    multi_action::claim_offer(alice, carol_address);
  }

  // Try to finalize account without governance
  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x30001, location = ol_framework::multi_action)]
  fun finalize_without_gov(root: &signer, alice: &signer) {
    let _vals = mock::genesis_n_vals(root, 1);
    multi_action::finalize_and_cage2(alice);
  }

  // Try to finalize account without offer
  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x60017, location = ol_framework::multi_action)]
  fun finalize_without_offer(root: &signer, alice: &signer) {
    let _vals = mock::genesis_n_vals(root, 1);
    multi_action::init_gov(alice);
    multi_action::finalize_and_cage2(alice);
  }

  // Try to finalize account without enough offer claimed
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  #[expected_failure(abort_code = 0x30018, location = ol_framework::multi_action)]
  fun finalize_without_enough_claimed(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    let alice_address = @0x1000a;
    multi_action::init_gov(alice);
    
    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(bob));
    vector::push_back(&mut authorities, signer::address_of(carol));
    multi_action::propose_offer(alice, authorities, option::none());

    // bob claim the offer
    multi_action::claim_offer(bob, alice_address);

    // finalize the multi_action account
    multi_action::finalize_and_cage2(alice);
  }

  // Try to finalize account already finalized
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  #[expected_failure(abort_code = 0x80019, location = ol_framework::multi_action)]
  fun finalize_already_finalized(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
    let _vals = mock::genesis_n_vals(root, 3);
    let alice_address = @0x1000a;
    multi_action::init_gov(alice);
    
    // invite the vals to the resource account
    let authorities = vector::empty<address>();
    vector::push_back(&mut authorities, signer::address_of(bob));
    vector::push_back(&mut authorities, signer::address_of(carol));
    multi_action::propose_offer(alice, authorities, option::none());

    // bob claim the offer
    multi_action::claim_offer(bob, alice_address);
    multi_action::claim_offer(carol, alice_address);

    // finalize the multi_action account
    multi_action::finalize_and_cage2(alice);
    multi_action::finalize_and_cage2(alice);
  }
  
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
