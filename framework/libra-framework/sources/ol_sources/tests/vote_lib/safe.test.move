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
  use ol_framework::ol_account;
  use aptos_framework::resource_account;
  use ol_framework::gas_coin;

  struct DummyType has drop, store {}

  #[test(root = @ol_framework, dave = @0x1000d)]
  fun init_safe(root: &signer, dave: &signer) {


    let vals = mock::genesis_n_vals(root, 2);
    mock::ol_initialize_coin(root);

    // the account first needs to be created as a "resource account"
    // previously in ol this meant to "Brick" the authkey. Similar technique as vendor.


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
}