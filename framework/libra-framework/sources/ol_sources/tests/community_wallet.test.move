
  #[test_only]
  module ol_framework::test_community_wallet{
    use ol_framework::community_wallet;
    use ol_framework::community_wallet_init;
    use ol_framework::donor_voice_txs;
    use ol_framework::mock;
    use ol_framework::ol_account;
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
        assert!(!ol_account::is_cage(community_wallet_address), 7357002);

        // fix it by calling multi auth:
        community_wallet_init::finalize_and_cage(community);
        // multi_action::finalize_and_cage(community);
        assert!(ol_account::is_cage(community_wallet_address), 7357003);

        community_wallet_init::assert_qualifies(community_wallet_address);
    }


    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 196618, location = 0x1::ol_account)]
    fun sponsor_cant_transfer(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer,) {
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

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, _carol = @0x1000c)]
    fun cw_init_with_n_below_minimum_sigs(root: &signer, alice: &signer, bob: &signer, _carol: &signer) {
        // A community wallet by default must be 2/3 multisig.
        // This test verifies that the wallet can not be initialized with less signers
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let signers = vector::empty<address>();

        // helpers in line to help
        vector::push_back(&mut signers, signer::address_of(bob));
        // vector::push_back(&mut signers, signer::address_of(carol));


        donor_voice_txs::make_donor_voice(alice, signers, 1);

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


  }