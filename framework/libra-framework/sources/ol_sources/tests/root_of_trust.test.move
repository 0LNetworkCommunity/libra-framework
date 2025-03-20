#[test_only]
module ol_framework::root_of_trust_tests {
    use std::vector;
    use diem_framework::account;
    use diem_framework::timestamp;
    use ol_framework::ancestry;
    use ol_framework::ol_account;
    use ol_framework::mock;
    use ol_framework::root_of_trust;

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

        // Create accounts first with ol_account
        ol_account::create_account(framework, ALICE_AT_GENESIS);
        ol_account::create_account(framework, BOB_ALICES_CHILD);
        ol_account::create_account(framework, CAROL_BOBS_CHILD);
        ol_account::create_account(framework, DAVE_AT_GENESIS);
        ol_account::create_account(framework, EVE_DAVES_CHILD);

        // Get signers for ancestry setup
        let alice_signer = account::create_signer_for_test(ALICE_AT_GENESIS);
        let bob_signer = account::create_signer_for_test(BOB_ALICES_CHILD);
        let carol_signer = account::create_signer_for_test(CAROL_BOBS_CHILD);
        let dave_signer = account::create_signer_for_test(DAVE_AT_GENESIS);
        let eve_signer = account::create_signer_for_test(EVE_DAVES_CHILD);

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

    #[test(framework = @0x1)]
    fun test_framework_root_init(framework: &signer) {
        // Setup test environment and accounts
        setup_test_ancestry(framework);

        // Create initial root of trust vector with genesis validators
        let roots = vector::empty();
        vector::push_back(&mut roots, ALICE_AT_GENESIS);
        vector::push_back(&mut roots, DAVE_AT_GENESIS);

        // Initialize root of trust on framework account
        root_of_trust::framework_migration(
            framework,
            roots,
            2,  // minimum_cohort size
            5, // days until next rotation
        );

        // Assert root of trust was initialized correctly
        assert!(root_of_trust::is_root_at_registry(@0x1, ALICE_AT_GENESIS), 1);
        assert!(root_of_trust::is_root_at_registry(@0x1, DAVE_AT_GENESIS), 2);

        // Verify non-root addresses are not recognized
        assert!(!root_of_trust::is_root_at_registry(@0x1, BOB_ALICES_CHILD), 3);
        assert!(!root_of_trust::is_root_at_registry(@0x1, EVE_DAVES_CHILD), 4);
    }

    #[test(framework = @0x1)]
    fun test_framework_root_rotation(framework: &signer) {
        // Setup test environment and accounts
        setup_test_ancestry(framework);

        // Initialize with original roots (Alice and Dave)
        let initial_roots = vector::empty();
        vector::push_back(&mut initial_roots, ALICE_AT_GENESIS);
        vector::push_back(&mut initial_roots, DAVE_AT_GENESIS);

        root_of_trust::framework_migration(
            framework,
            initial_roots,
            2,  // minimum_cohort size
            5, // days until next rotation
        );

        // Verify initial setup
        assert!(root_of_trust::is_root_at_registry(@0x1, ALICE_AT_GENESIS), 1);
        assert!(root_of_trust::is_root_at_registry(@0x1, DAVE_AT_GENESIS), 2);

        // Try to rotate before window - should fail
        let adds = vector::singleton(BOB_ALICES_CHILD);
        let removes = vector::singleton(ALICE_AT_GENESIS);
        assert!(!root_of_trust::can_rotate(@0x1), 3);

        // Advance time past rotation window (7 days)
        timestamp::fast_forward_seconds(7 * 24 * 60 * 60);

        // Verify rotation is now possible
        assert!(root_of_trust::can_rotate(@0x1), 4);

        // Perform rotation
        root_of_trust::rotate_roots(framework, adds, removes);

        // Verify new root set
        assert!(!root_of_trust::is_root_at_registry(@0x1, ALICE_AT_GENESIS), 5);
        assert!(root_of_trust::is_root_at_registry(@0x1, BOB_ALICES_CHILD), 6);
        assert!(root_of_trust::is_root_at_registry(@0x1, DAVE_AT_GENESIS), 7);
    }
}
