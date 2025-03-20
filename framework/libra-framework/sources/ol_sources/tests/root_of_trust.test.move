#[test_only]
module ol_framework::root_of_trust_tests {
    use ol_framework::ancestry;
    use ol_framework::ol_account;
    use ol_framework::mock;

    // Test addresses with descriptive names showing relationships
    const ALICE_AT_GENESIS: address = @0x123;
    const BOB_ALICES_CHILD: address = @0x456;
    const CAROL_BOBS_CHILD: address = @0x789;
    const DAVE_AT_GENESIS: address = @0x321;
    const EVE_DAVES_CHILD: address = @0x654;

    #[test_only]
    /// Sets up test environment with accounts and ancestry relationships:
    /// ALICE_AT_GENESIS -> BOB_ALICES_CHILD -> CAROL_BOBS_CHILD
    /// DAVE_AT_GENESIS -> EVE_DAVES_CHILD
    fun setup_test_ancestry(framework: &signer) {
        // Initialize test environment first
        mock::ol_test_genesis(framework);

        // Create accounts first
        let alice_signer = ol_account::create_account(framework, ALICE_AT_GENESIS);
        let bob_signer = ol_account::create_account(framework, BOB_ALICES_CHILD);
        let carol_signer = ol_account::create_account(framework, CAROL_BOBS_CHILD);
        let dave_signer = ol_account::create_account(framework, DAVE_AT_GENESIS);
        let eve_signer = ol_account::create_account(framework, EVE_DAVES_CHILD);

        // Set up ancestry relationships
        ancestry::adopt_this_child(&alice_signer, &bob_signer);
        ancestry::adopt_this_child(&bob_signer, &carol_signer);
        ancestry::adopt_this_child(&dave_signer, &eve_signer);
    }

    #[test(framework = @0x1)]
    fun test_basic_ancestry_setup(framework: &signer) {
        setup_test_ancestry(framework);

        // Verify direct relationships
        assert!(ancestry::is_in_tree(ALICE_AT_GENESIS, BOB_ALICES_CHILD), 1);
        assert!(ancestry::is_in_tree(BOB_ALICES_CHILD, CAROL_BOBS_CHILD), 2);
        assert!(ancestry::is_in_tree(DAVE_AT_GENESIS, EVE_DAVES_CHILD), 3);

        // Verify multi-generation relationship
        assert!(ancestry::is_in_tree(ALICE_AT_GENESIS, CAROL_BOBS_CHILD), 4);
    }
}
