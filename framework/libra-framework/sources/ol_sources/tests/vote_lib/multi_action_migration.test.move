#[test_only]
module ol_framework::test_multi_action_migration {
    use ol_framework::mock;
    use ol_framework::multi_action;
    use ol_framework::multi_action_migration;
    use std::signer;
    use std::option;
    use std::vector;

    // print
    // use std::debug::print;

    // Happy Day: init offer on a legacy multisign initiated account
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    fun migrate_offer_account_initialized(root: &signer, alice: &signer) {
        let _vals = mock::genesis_n_vals(root, 2);
        let alice_address = signer::address_of(alice);

        // initialize the multi_action account
        multi_action::init_gov_deprecated(alice);
        assert!(multi_action::exists_offer(alice_address) == false, 7357001);

        // migrate the offer
        multi_action_migration::migrate_offer(alice, alice_address);
        assert!(multi_action::exists_offer(alice_address) == true, 7357002);
        
        // offer authority to bob
        multi_action::propose_offer(alice, vector::singleton(@0x1000b), option::some(9));

        // check the offer is proposed
        assert!(multi_action::get_offer_proposed(alice_address) == vector::singleton(@0x1000b), 7357003);
        assert!(multi_action::get_offer_claimed(alice_address) == vector::empty(), 7357004);
        assert!(multi_action::get_offer_expiration_epoch(alice_address) == vector::singleton(9), 7357005);
    }

    // Happy Day: init offer on a legacy mulstisign finalized account
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c, dave = @0x1000d)]
    fun migrate_offer_account_finalized(root: &signer, alice: &signer, bob: &signer, carol: &signer, dave: &signer) {
        let _vals = mock::genesis_n_vals(root, 4);
        let alice_address = signer::address_of(alice);

        // make alice account multisign with deprecated methods
        multi_action::init_gov_deprecated(alice);
        let authorities = vector::singleton(@0x1000b);
        vector::push_back(&mut authorities, @0x1000c);
        multi_action::finalize_and_cage_deprecated(alice, authorities, 2);
        assert!(multi_action::exists_offer(alice_address) == false, 7357001);

        // carol migrate the offer
        multi_action_migration::migrate_offer(carol, alice_address);
        assert!(multi_action::exists_offer(alice_address) == true, 7357002);

        // carol propose dave as new authority
        let id = multi_action::propose_governance(carol, alice_address,
        vector::singleton(@0x1000d), true, option::none(),
        option::none());

        // bob votes
        let passed = multi_action::vote_governance(bob, alice_address, &id);
        assert!(passed, 7357003);

        // dave claim the offer
        multi_action::claim_offer(dave, alice_address);

        // check new authorities
        let authorities = vector::singleton(@0x1000b);
        vector::push_back(&mut authorities, @0x1000c);
        vector::push_back(&mut authorities, @0x1000d);
        assert!(multi_action::get_authorities(alice_address) == authorities, 7357004);
    }

    // Try to migrate offer of an account already with offer
    #[test(root = @ol_framework, alice = @0x1000a)]
    #[expected_failure(abort_code = 0x80001, location = ol_framework::multi_action_migration)]
    fun migrate_offer_account_with_offer(root: &signer, alice: &signer) {
        let _vals = mock::genesis_n_vals(root, 2);
        let alice_address = signer::address_of(alice);

        // initialize the multi_action account
        multi_action::init_gov(alice);

        // try to migrate offer
        multi_action_migration::migrate_offer(alice, alice_address);
    }

    // Try to migrate offer of an account without governance initiated
    #[test(root = @ol_framework, alice = @0x1000a)]
    #[expected_failure(abort_code = 0x30003, location = ol_framework::multi_action_migration)]
    fun migrate_offer_account_without_gov(root: &signer, alice: &signer) {
        let _vals = mock::genesis_n_vals(root, 1);
        let alice_address = signer::address_of(alice);

        // try to migrate offer
        multi_action_migration::migrate_offer(alice, alice_address);
    }

    // Try to migrate offer through someone without authority
    #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
    #[expected_failure(abort_code = 0x50002, location = ol_framework::multi_action_migration)]
    fun migrate_offer_without_authority(root: &signer, alice: &signer, bob: &signer) {
        let _vals = mock::genesis_n_vals(root, 2);
        let alice_address = signer::address_of(alice);

        // initialize the multi_action account
        multi_action::init_gov_deprecated(alice);

        // try to migrate offer
        multi_action_migration::migrate_offer(bob, alice_address);
    }
}