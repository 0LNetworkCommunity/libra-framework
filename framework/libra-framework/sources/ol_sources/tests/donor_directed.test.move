
#[test_only]

module ol_framework::test_donor_directed {
  use ol_framework::donor_directed;
  use ol_framework::mock;
  use ol_framework::ol_account;
  use aptos_framework::resource_account;

  use std::vector;
  use std::signer;

    #[test(root = @ol_framework, alice = @0x1000a)]
    fun init_donor_directed(root: &signer, alice: &signer) {
      mock::genesis_n_vals(root, 4);

      // Alice turns this account into a resource account. This can also happen after other donor directed initializations happen (or deposits). But must be complete before the donor directed wallet can be used.
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let new_resource_address = signer::address_of(&resource_sig);
      assert!(resource_account::is_resource_account(new_resource_address), 7357003);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);


      let list = donor_directed::get_root_registry();
      assert!(vector::length(&list) == 1, 7357001);

      assert!(donor_directed::is_donor_directed(new_resource_address), 7357002);

    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun dd_propose_payment(root: &signer, alice: &signer, bob: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let new_resource_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    fun schedule_happy(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
      // Scenario: Alice creates a resource_account which will be a donor directed account. She will not be one of the authorities of the account.
      // only bob, carol, and dave with be authorities

      mock::genesis_n_vals(root, 4);
      let (resource_sig, _cap) = ol_account::ol_create_resource_account(alice, b"0x1");
      let new_resource_address = signer::address_of(&resource_sig);

      // the account needs basic donor directed structs
      donor_directed::init_donor_directed(&resource_sig, @0x1000b, @0x1000c, @0x1000d, 2);

      let uid = donor_directed::propose_payment(bob, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(!completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(!donor_directed::is_scheduled(new_resource_address, &uid), 7357008);

      let uid = donor_directed::propose_payment(carol, new_resource_address, @0x1000b, 100, b"thanks bob");
      let (found, idx, status_enum, completed) = donor_directed::get_multisig_proposal_state(new_resource_address, &uid);
      assert!(found, 7357004);
      assert!(idx == 0, 7357005);
      assert!(status_enum == 1, 7357006);
      assert!(completed, 7357007);

      // it is not yet scheduled, it's still only a proposal by an admin
      assert!(donor_directed::is_scheduled(new_resource_address, &uid), 7357008);
    }


    //   let uid = DonorDirected::propose_payment(&sender, @Alice, @Bob, 100, b"thanks bob");
    //   let (found, idx, status_enum, completed) = DonorDirected::get_multisig_proposal_state(@Alice, &uid);

    //   assert!(found, 7357004);
    //   assert!(idx == 0, 7357005);
    //   assert!(status_enum == 1, 7357006);
    //   assert!(completed, 7357007);

    //   // Now it's scheduled since we got over the threshold
    //   assert!(DonorDirected::is_scheduled(@Alice, &uid), 7357003);

    //   // the default timed payment is 3 epochs, we are in epoch 1
    //   let list = DonorDirected::find_by_deadline(@Alice, 4);
    //   assert!(Vector::contains(&list, &uid), 7357008);
    // }

}

