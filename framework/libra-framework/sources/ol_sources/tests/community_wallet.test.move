
  #[test_only]
  module ol_framework::test_community_wallet{
    use ol_framework::community_wallet;
    use ol_framework::community_wallet_init;
    use ol_framework::donor_voice_txs;
    use diem_framework::multisig_account;
    use ol_framework::mock;
    use ol_framework::ol_account;
    use ol_framework::ancestry;
    use std::signer;
    use std::vector;


  // use diem_std::debug::print;


    #[test(root = @ol_framework, community = @0x10011)]
    fun migrate_cw_bug_not_resource(root: &signer, community: &signer) {

        // create genesis and fund accounts
        let auths = mock::genesis_n_vals(root, 3);
        mock::ol_initialize_coin_and_fund_vals(root, 10000000, true);


        let community_wallet_address = signer::address_of(community);
        // genesis migration would have created this account.
        ol_account::create_account(root, community_wallet_address);
        // migrate community wallet
        // THIS PUTS THE ACCOUNT IN LIMBO
        community_wallet_init::migrate_community_wallet_account(root, community);

        // verify correct migration of community wallet
        assert!(community_wallet::is_init(community_wallet_address), 7357001); //TODO: find appropriate error codes

        // the usual initialization should fix the structs
        community_wallet_init::init_community(community, auths);
        // confirm the bug
        assert!(!multisig_account::is_multisig(community_wallet_address), 7357002);

        // fix it by calling multi auth:
        community_wallet_init::finalize_and_cage(community, auths, vector::length(&auths));
        // multi_action::finalize_and_cage(community);
        assert!(multisig_account::is_multisig(community_wallet_address), 7357003);

        community_wallet_init::assert_qualifies(community_wallet_address);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 196618, location = 0x1::ol_account)]
    fun cw_sponsor_cant_transfer(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer,) {
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let signers = vector::empty<address>();

        // helpers in line to help
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(carol));
        vector::push_back(&mut signers, signer::address_of(dave));

        // Alice is a vanilla account and should be able to transfer
        ol_account::transfer(alice, @0x1000b, 100);

        community_wallet_init::init_community(alice, signers);

        // After being set as a community wallet, the owner loses control over the wallet
        ol_account::transfer(alice, @0x1000b, 100);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
    #[expected_failure(abort_code = 65545, location = 0x1::community_wallet_init)]
    fun cw_init_below_minimum_sigs(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be initialized with less signers
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let signers = vector::empty<address>();

        // helpers in line to help
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(carol));

        community_wallet_init::init_community(alice, signers);

    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    #[expected_failure(abort_code = 65541, location = 0x1::community_wallet_init)]
    fun cw_decrease_below_m_authorized_sigs(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer ) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be initialized with less signers
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let (_, carol_balance_pre) = ol_account::balance(@0x1000c);
        assert!(carol_balance_pre == 1000, 7357001);

        let bob_addr = signer::address_of(bob);
        let dave_addr = signer::address_of(dave);
        let eve_addr = signer::address_of(eve);

        ancestry::fork_migrate(root, bob, vector::singleton(bob_addr));
        ancestry::fork_migrate(root, dave, vector::singleton(dave_addr));
        ancestry::fork_migrate(root, eve, vector::singleton(eve_addr));

        let signers = vector::empty<address>();

        // helpers in line to help
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(dave));
        vector::push_back(&mut signers, signer::address_of(eve));

        community_wallet_init::init_community(alice, signers);

        // fix it by calling multi auth:
        community_wallet_init::finalize_and_cage(alice, signers, 2);

        let alice_comm_wallet_addr = signer::address_of(alice);
        let carols_addr = signer::address_of(carol);


        // VERIFY PAYMENTS OPERATE AS EXPECTED
        let uid = donor_voice_txs::propose_payment(bob, alice_comm_wallet_addr, carols_addr, 100, b"thanks carol");
        let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
        assert!(found, 7357004);
        assert!(idx == 0, 7357005);
        assert!(status_enum == 1, 7357006);
        assert!(!completed, 7357007);

        // it is not yet scheduled, it's still only a proposal by an admin
        assert!(!donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);

        let uid = donor_voice_txs::propose_payment(dave, alice_comm_wallet_addr, @0x1000c, 100, b"thanks carol");
        let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
        assert!(found, 7357004);
        assert!(idx == 0, 7357005);
        assert!(status_enum == 1, 7357006);
        assert!(completed, 7357007); // now completed

        // confirm it is scheduled
        assert!(donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);

        // PROCESS THE PAYMENT
        // the default timed payment is 3 epochs, we are in epoch 1
        let list = donor_voice_txs::find_by_deadline(alice_comm_wallet_addr, 3);
        assert!(vector::contains(&list, &uid), 7357009);

        // process epoch 3 accounts
        donor_voice_txs::process_donor_voice_accounts(root, 3);

        let (_, carol_balance) = ol_account::balance(@0x1000c);
        assert!(carol_balance > carol_balance_pre, 7357005);
        assert!(carol_balance == 1100, 7357006);

        // remove a signer and decrease to 2 and verify the community wallet is bricked
        // signers must be removed by n signers
        community_wallet_init::change_signer_community_multisig(bob, alice_comm_wallet_addr, dave_addr, false, 2, 10); // remove by setting as false
        community_wallet_init::change_signer_community_multisig(eve, alice_comm_wallet_addr, dave_addr, false, 2, 10);

    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    #[expected_failure(abort_code = 65545, location = 0x1::community_wallet_init)]
    fun cw_decrease_below_minimum_n_sigs(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer ) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be initialized with less signers
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let (_, carol_balance_pre) = ol_account::balance(@0x1000c);
        assert!(carol_balance_pre == 1000, 7357001);

        let bob_addr = signer::address_of(bob);
        let dave_addr = signer::address_of(dave);
        let eve_addr = signer::address_of(eve);

        ancestry::fork_migrate(root, bob, vector::singleton(bob_addr));
        ancestry::fork_migrate(root, dave, vector::singleton(dave_addr));
        ancestry::fork_migrate(root, eve, vector::singleton(eve_addr));

        let signers = vector::empty<address>();

        // helpers in line to help
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(dave));
        vector::push_back(&mut signers, signer::address_of(eve));

        community_wallet_init::init_community(alice, signers);

        // fix it by calling multi auth:
        community_wallet_init::finalize_and_cage(alice, signers, 2);

        let alice_comm_wallet_addr = signer::address_of(alice);
        let carols_addr = signer::address_of(carol);


        // VERIFY PAYMENTS OPERATE AS EXPECTED
        let uid = donor_voice_txs::propose_payment(bob, alice_comm_wallet_addr, carols_addr, 100, b"thanks carol");
        let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
        assert!(found, 7357004);
        assert!(idx == 0, 7357005);
        assert!(status_enum == 1, 7357006);
        assert!(!completed, 7357007);

        // it is not yet scheduled, it's still only a proposal by an admin
        assert!(!donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);

        let uid = donor_voice_txs::propose_payment(dave, alice_comm_wallet_addr, @0x1000c, 100, b"thanks carol");
        let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
        assert!(found, 7357004);
        assert!(idx == 0, 7357005);
        assert!(status_enum == 1, 7357006);
        assert!(completed, 7357007); // now completed

        // confirm it is scheduled
        assert!(donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);

        // PROCESS THE PAYMENT
        // the default timed payment is 3 epochs, we are in epoch 1
        let list = donor_voice_txs::find_by_deadline(alice_comm_wallet_addr, 3);
        assert!(vector::contains(&list, &uid), 7357009);

        // process epoch 3 accounts
        donor_voice_txs::process_donor_voice_accounts(root, 3);

        let (_, carol_balance) = ol_account::balance(@0x1000c);
        assert!(carol_balance > carol_balance_pre, 7357005);
        assert!(carol_balance == 1100, 7357006);

        // remove a signer and decrease to 2 and verify the community wallet is bricked
        // signers must be removed by n signers
        community_wallet_init::change_signer_community_multisig(bob, alice_comm_wallet_addr, dave_addr, false, 1, 10); // remove by setting as false



    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    #[expected_failure(abort_code = 65542, location = 0x1::community_wallet_init)]
    fun cw_init_with_n_and_m_below_minimum_sigs(root: &signer, alice: &signer, bob: &signer) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be initialized with less signers
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);


        // create signers
        let signers = vector::empty<address>();
        vector::push_back(&mut signers, signer::address_of(bob));


        community_wallet_init::migrate_community_wallet_account(root, alice);

        donor_voice_txs::make_donor_voice(alice, signers, 1);

        // try to cage the address by calling multi auth
        community_wallet_init::finalize_and_cage(alice, signers, 1);

    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 65542, location = 0x1::community_wallet_init)]
    fun cw_dd_init_with_n_below_minimum_sigs(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be decreased below this specification
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);


        // create signers
        let signers = vector::empty<address>();
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(carol));
        vector::push_back(&mut signers, signer::address_of(dave));

        community_wallet_init::migrate_community_wallet_account(root, alice);

        donor_voice_txs::make_donor_voice(alice, signers, 1);

        // try to cage the address by calling multi auth
        community_wallet_init::finalize_and_cage(alice, signers, 1);

    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 65542, location = 0x1::community_wallet_init)]
    fun cw_dd_decrease_m_below_minimum_sigs(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be decreased below this specification
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);


        // create signers
        let signers = vector::empty<address>();
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(carol));
        vector::push_back(&mut signers, signer::address_of(dave));

        community_wallet_init::migrate_community_wallet_account(root, alice);

        donor_voice_txs::make_donor_voice(alice, signers, 1);

        // try to cage the address by calling multi auth
        community_wallet_init::finalize_and_cage(alice, signers, 1);

    }


    // #[test(root = @ol_framework, _alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    // #[expected_failure(abort_code = 196618, location = 0x1::ol_account)]
    // fun cw_below_minimum_signer(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer) {
    //     // A community wallet by default must be 2/3 multisig.
    //     // This test verifies that the wallet can not be initialized with less signers
    //     mock::genesis_n_vals(root, 4);
    //     mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

    //     let signers = vector::empty<address>();

    //     // helpers in line to help
    //     vector::push_back(&mut signers, signer::address_of(bob));
    //     vector::push_back(&mut signers, signer::address_of(dave));

    //     community_wallet_init::init_community(alice, signers);

    //     // After being set as a community wallet, the owner loses control over the wallet
    //     ol_account::transfer(alice, @0x1000b, 100);
    // }

        //      let alice_comm_wallet_addr = signer::address_of(alice);
    //    let carols_addr = signer::address_of(carol);

    //     let uid = donor_voice_txs::propose_payment(bob, alice_comm_wallet_addr, carols_addr, 100, b"thanks carol");
    //     let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
    //     assert!(found, 7357004);
    //     assert!(idx == 0, 7357005);
    //     assert!(status_enum == 1, 7357006);
    //     assert!(!completed, 7357007);

    //   // it is not yet scheduled, it's still only a proposal by an admin
    //   assert!(!donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);


  }
