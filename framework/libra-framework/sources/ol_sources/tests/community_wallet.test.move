
  #[test_only]
  module ol_framework::test_community_wallet{
    use ol_framework::ballot;
    use ol_framework::community_wallet;
    use ol_framework::community_wallet_init;
    use ol_framework::donor_voice_txs;
    use ol_framework::multi_action;
    use diem_framework::multisig_account;
    use ol_framework::mock;
    use ol_framework::ol_account;
    use ol_framework::ancestry;
    use std::signer;
    use std::vector;

    // use diem_std::debug::print;

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, community = @0x10011)]
    fun migrate_cw_bug_not_resource(root: &signer, alice: &signer, bob: &signer, carol: &signer, community: &signer) {

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
        community_wallet_init::init_community(community, auths, 2);
        // confirm the bug
        assert!(!multisig_account::is_multisig(community_wallet_address), 7357002);

        // vals claim the offer
        multi_action::claim_offer(alice, community_wallet_address);
        multi_action::claim_offer(bob, community_wallet_address);
        multi_action::claim_offer(carol, community_wallet_address);

        // fix it by calling multi auth:
        community_wallet_init::finalize_and_cage(community, vector::length(&auths));
        // multi_action::finalize_and_cage(community);
        assert!(multisig_account::is_multisig(community_wallet_address), 7357003);

        community_wallet_init::assert_qualifies(community_wallet_address);
    }

    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 196620, location = 0x1::ol_account)]
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

        community_wallet_init::init_community(alice, signers, 2);

        // After being set as a community wallet, the owner loses control over the wallet
        ol_account::transfer(alice, @0x1000b, 100);
    }

    // Test payment proposal and processing
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    fun cw_payment_proposal(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer, eve: &signer ) {
        mock::genesis_n_vals(root, 5);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        // initilize accounts
        let (_, carol_balance_pre) = ol_account::balance(@0x1000c);
        assert!(carol_balance_pre == 1000, 7357001);
        let bob_addr = signer::address_of(bob);
        let dave_addr = signer::address_of(dave);
        let eve_addr = signer::address_of(eve);
        ancestry::test_fork_migrate(root, bob, vector::singleton(bob_addr));
        ancestry::test_fork_migrate(root, dave, vector::singleton(dave_addr));
        ancestry::test_fork_migrate(root, eve, vector::singleton(eve_addr));

        // setup community wallet
        community_wallet_init::init_community(alice, vector[bob_addr,dave_addr,eve_addr], 2);
        multi_action::claim_offer(bob, signer::address_of(alice));
        multi_action::claim_offer(dave, signer::address_of(alice));
        multi_action::claim_offer(eve, signer::address_of(alice));
        community_wallet_init::finalize_and_cage(alice, 2);

        let alice_comm_wallet_addr = signer::address_of(alice);
        let carols_addr = signer::address_of(carol);

        // VERIFY PAYMENTS OPERATE AS EXPECTED
        // bob propose payment
        let uid = donor_voice_txs::test_propose_payment(bob, alice_comm_wallet_addr, carols_addr, 100, b"thanks carol");
        let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
        assert!(found, 7357004);
        assert!(idx == 0, 7357005);
        assert!(status_enum == ballot::get_pending_enum(), 7357006);
        assert!(!completed, 7357007);

        // it is not yet scheduled, it's still only a proposal by an admin
        assert!(!donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);

        // dave votes the payment and it is approved.
        let uid = donor_voice_txs::test_propose_payment(dave, alice_comm_wallet_addr, @0x1000c, 100, b"thanks carol");
        let (found, idx, status_enum, completed) = donor_voice_txs::get_multisig_proposal_state(alice_comm_wallet_addr, &uid);
        assert!(found, 7357004);
        assert!(idx == 0, 7357005);
        assert!(status_enum == ballot::get_approved_enum(), 7357006);
        assert!(completed, 7357007); // now completed

        // confirm it is scheduled
        assert!(donor_voice_txs::is_scheduled(alice_comm_wallet_addr, &uid), 7357008);

        // PROCESS THE PAYMENT
        // the default timed payment is 3 epochs, we are in epoch 1
        let list = donor_voice_txs::find_by_deadline(alice_comm_wallet_addr, 3);
        assert!(vector::contains(&list, &uid), 7357009);

        // process epoch 3 accounts
        donor_voice_txs::process_donor_voice_accounts(root, 3);

        // verify the payment was processed
        let (_, carol_balance) = ol_account::balance(@0x1000c);
        assert!(carol_balance > carol_balance_pre, 7357005);
        assert!(carol_balance == 1100, 7357006);
    }

    // Try to initialize with less than the required signitures
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    #[expected_failure(abort_code = 0x1000B, location = 0x1::community_wallet_init)]
    fun cw_init_with_less_signitures_than_min(root: &signer, alice: &signer) {
        // A community wallet by default must be 2/3 multisig.
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        // try to initialize
        let authorities = vector::singleton(@0x1000b);
        vector::push_back(&mut authorities, @0x1000c);
        vector::push_back(&mut authorities, @0x1000d);
        community_wallet_init::init_community(alice, authorities, 1);
    }

    // Try to initialize with less than the required authorities
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x1000A, location = 0x1::community_wallet_init)]
    fun cw_init_with_less_authorities_than_min(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
        // A community wallet by default must be 2/3 multisig.
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        // try to initialize
        let signers = vector::empty<address>();
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(carol));
        community_wallet_init::init_community(alice, signers, 2);
    }

    // Try to finalize with less than the required authorities
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x10006, location = 0x1::community_wallet_init)]
    fun cw_finalize_with_less_authorities_than_min(root: &signer, alice: &signer, bob: &signer, carol: &signer) {
        // A community wallet by default must be 2/3 multisig.
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let authorities = vector::singleton(@0x1000b);
        vector::push_back(&mut authorities, @0x1000c);
        vector::push_back(&mut authorities, @0x1000d);
        community_wallet_init::init_community(alice, authorities, 2);

        multi_action::claim_offer(bob, signer::address_of(alice));
        multi_action::claim_offer(carol, signer::address_of(alice));

        // try to finalize
        community_wallet_init::finalize_and_cage(alice, 2);
    }

    // Try to finalize with less than the required signitures
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x10006, location = 0x1::community_wallet_init)]
    fun cw_finalize_with_less_signitures_than_min(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
        // A community wallet by default must be 2/3 multisig.
        mock::genesis_n_vals(root, 4);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let signers = vector::empty<address>();
        vector::push_back(&mut signers, signer::address_of(bob));
        vector::push_back(&mut signers, signer::address_of(carol));
        vector::push_back(&mut signers, signer::address_of(dave));

        community_wallet_init::init_community(alice, signers, 2);

        multi_action::claim_offer(bob, signer::address_of(alice));
        multi_action::claim_offer(carol, signer::address_of(alice));
        multi_action::claim_offer(dave, signer::address_of(alice));

        // try to finalize
        community_wallet_init::finalize_and_cage(alice, 1);
    }

    // change the authorities offer of a community wallet
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d, eve = @0x1000e)]
    fun cw_change_authorities(root: &signer, alice: &signer, bob: &signer, carol: &signer,dave: &signer, eve: &signer) {
        mock::genesis_n_vals(root, 5);
        mock::ol_initialize_coin_and_fund_vals(root, 1000, true);

        let authorities = vector::empty<address>();
        vector::push_back(&mut authorities, signer::address_of(bob));
        vector::push_back(&mut authorities, signer::address_of(carol));
        vector::push_back(&mut authorities, signer::address_of(dave));
        community_wallet_init::init_community(alice, authorities, 2);

        vector::pop_back(&mut authorities); // remove dave
        vector::push_back(&mut authorities, signer::address_of(eve)); // add eve
        community_wallet_init::propose_offer(alice, authorities, 2);

        multi_action::claim_offer(bob, signer::address_of(alice));
        multi_action::claim_offer(carol, signer::address_of(alice));
        multi_action::claim_offer(eve, signer::address_of(alice));

        community_wallet_init::finalize_and_cage(alice, 2);

        // certify the change
        let new_authorities = multi_action::get_authorities(signer::address_of(alice));
        assert!(vector::length(&new_authorities) == 3, 7357001);
        assert!(vector::contains(&new_authorities, &signer::address_of(bob)), 7357002);
        assert!(vector::contains(&new_authorities, &signer::address_of(carol)), 7357003);
        assert!(vector::contains(&new_authorities, &signer::address_of(eve)), 7357004);
    }

    // Try to propose offer with less authorities than the minimum
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x1000A, location = 0x1::community_wallet_init)]
    fun cw_propose_offer_with_less_authorities_than_min(root: &signer, alice: &signer) {
        mock::genesis_n_vals(root, 4);
        community_wallet_init::init_community(alice, vector[@0x1000b, @0x1000c, @0x1000d], 2);
        community_wallet_init::propose_offer(alice, vector[@0x1000b, @0x1000c], 2);
    }

    // Try to propose offer with less signatures than the minimum
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x1000B, location = 0x1::community_wallet_init)]
    fun cw_propose_offer_with_less_signatures_than_min(root: &signer, alice: &signer) {
        mock::genesis_n_vals(root, 4);
        community_wallet_init::init_community(alice, vector[@0x1000b, @0x1000c, @0x1000d], 2);
        community_wallet_init::propose_offer(alice, vector[@0x1000b, @0x1000c], 1);
    }

    // Try to reduce the number of signatures below the minimum
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x1000B, location = 0x1::community_wallet_init)]
    fun cw_decrease_signatures_below_minimum(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
        mock::genesis_n_vals(root, 5);
        let alice_address = signer::address_of(alice); // community wallet address

        // 1. Initializes a community wallet with 3 authorities and 2 signatures.
        let authorities = vector[@0x1000b, @0x1000c, @0x1000d];
        community_wallet_init::init_community(alice, authorities, 2);
        multi_action::claim_offer(bob, alice_address);
        multi_action::claim_offer(carol, alice_address);
        multi_action::claim_offer(dave, alice_address);
        community_wallet_init::finalize_and_cage(alice, 2);
        let (num_signatures, _) = multi_action::get_threshold(alice_address);
        assert!(num_signatures == 2, 73573001);

        // 2. Try to change the requirement to 1 signature when adding eve
        community_wallet_init::change_signer_community_multisig(bob, alice_address, @0x1000e, true, 1, 10);
    }

    // Try to reduce the number of authorities below the minimum
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    #[expected_failure(abort_code = 0x1000A, location = 0x1::community_wallet_init)]
    fun cw_decrease_authorities_below_minimum(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
        mock::genesis_n_vals(root, 5);
        let alice_address = signer::address_of(alice); // community wallet address

        // 1. Initializes a community wallet with 3 authorities and 2 signatures.
        let authorities = vector[@0x1000b, @0x1000c, @0x1000d];
        community_wallet_init::init_community(alice, authorities, 2);
        multi_action::claim_offer(bob, alice_address);
        multi_action::claim_offer(carol, alice_address);
        multi_action::claim_offer(dave, alice_address);
        community_wallet_init::finalize_and_cage(alice, 2);
        let (num_signatures, _) = multi_action::get_threshold(alice_address);
        assert!(num_signatures == 2, 73573001);

        // 2. Try to remove authorities below the minimum
        community_wallet_init::change_signer_community_multisig(bob, alice_address, @0x1000b, false, 2, 10);
    }
}
