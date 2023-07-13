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
  use std::signer;
  use ol_framework::ol_account;
  use aptos_framework::resource_account;

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
}