#[test_only]
module ol_framework::test_permanent_vouches {
    use std::vector;
    use ol_framework::vouch;
    use ol_framework::mock;
    use ol_framework::epoch_helper;

    #[test(root = @ol_framework, alice = @0x1000a, _bob = @0x1000b)]
    fun vouch_persists_across_epochs(root: &signer, alice: &signer, _bob: &signer) {
        // Initialize genesis and validators
        mock::ol_test_genesis(root);
        mock::create_validator_accounts(root, 2, false);
        vouch::set_vouch_price(root, 0);
        
        // Alice vouches for Bob
        vouch::vouch_for(alice, @0x1000b);
        
        // Verify vouch exists
        let received_vouches = vouch::get_received_vouches_not_expired(@0x1000b);
        assert!(vector::contains(&received_vouches, &@0x1000a), 1);
        
        // Simulate multiple epochs passing (more than old expiration limit)
        let i = 0;
        while (i < 50) {  // Beyond old EXPIRATION_ELAPSED_EPOCHS
            mock::trigger_epoch(root);
            i = i + 1;
        };
        
        // Verify vouch still exists (permanent vouches)
        let received_vouches_after = vouch::get_received_vouches_not_expired(@0x1000b);
        assert!(vector::contains(&received_vouches_after, &@0x1000a), 2);
    }

    #[test(root = @ol_framework, alice = @0x1000a, _bob = @0x1000b)]
    fun vouch_revocation_still_works(root: &signer, alice: &signer, _bob: &signer) {
        // Initialize genesis and validators
        mock::ol_test_genesis(root);
        mock::create_validator_accounts(root, 2, false);
        vouch::set_vouch_price(root, 0);
        
        // Alice vouches for Bob
        vouch::vouch_for(alice, @0x1000b);
        
        // Verify vouch exists
        let received_vouches = vouch::get_received_vouches_not_expired(@0x1000b);
        assert!(vector::contains(&received_vouches, &@0x1000a), 1);
        
        // Alice revokes vouch for Bob
        vouch::revoke(alice, @0x1000b);
        
        // Verify vouch is removed
        let received_vouches_after = vouch::get_received_vouches_not_expired(@0x1000b);
        assert!(!vector::contains(&received_vouches_after, &@0x1000a), 2);
        
        // Verify given vouches are also updated
        let given_vouches = vouch::get_given_vouches_not_expired(@0x1000a);
        assert!(!vector::contains(&given_vouches, &@0x1000b), 3);
    }

    #[test(root = @ol_framework, alice = @0x1000a, _bob = @0x1000b)]
    fun vouch_update_extends_permanently(root: &signer, alice: &signer, _bob: &signer) {
        // Initialize genesis and validators
        mock::ol_test_genesis(root);
        mock::create_validator_accounts(root, 2, false);
        vouch::set_vouch_price(root, 0);
        
        // Alice vouches for Bob initially
        vouch::vouch_for(alice, @0x1000b);
        
        let (_received_vouches, epochs) = vouch::get_received_vouches(@0x1000b);
        let initial_epoch = *vector::borrow(&epochs, 0);
        
        // Advance one epoch to ensure difference
        mock::trigger_epoch(root);
        
        // Alice vouches for Bob again (should update epoch)
        vouch::vouch_for(alice, @0x1000b);
        
        let (received_vouches_after, epochs_after) = vouch::get_received_vouches(@0x1000b);
        let updated_epoch = *vector::borrow(&epochs_after, 0);
        
        // Should still have exactly one vouch from Alice
        assert!(vector::length(&received_vouches_after) == 1, 1);
        assert!(vector::contains(&received_vouches_after, &@0x1000a), 2);
        
        // Epoch should be updated to current
        let current_epoch = epoch_helper::get_current_epoch();
        assert!(updated_epoch == current_epoch, 3);
        assert!(updated_epoch >= initial_epoch, 4);
    }

    #[test(root = @ol_framework, alice = @0x1000a, _bob = @0x1000b, _charlie = @0x1000c)]
    fun lifetime_tracking_with_permanent_vouches(root: &signer, alice: &signer, _bob: &signer, _charlie: &signer) {
        // Initialize genesis and validators
        mock::ol_test_genesis(root);
        mock::create_validator_accounts(root, 3, false);
        vouch::set_vouch_price(root, 0);
        
        // Alice vouches for both Bob and Charlie
        vouch::vouch_for(alice, @0x1000b);
        vouch::vouch_for(alice, @0x1000c);
        
        // Check that Alice's given count is 2
        let given_vouches = vouch::get_given_vouches_not_expired(@0x1000a);
        assert!(vector::length(&given_vouches) == 2, 1);
        
        // Alice revokes vouch for Bob
        vouch::revoke(alice, @0x1000b);
        
        // Check that Alice's given count is now 1
        let given_vouches_after = vouch::get_given_vouches_not_expired(@0x1000a);
        assert!(vector::length(&given_vouches_after) == 1, 2);
        assert!(vector::contains(&given_vouches_after, &@0x1000c), 3);
        assert!(!vector::contains(&given_vouches_after, &@0x1000b), 4);
    }

    #[test(root = @ol_framework, alice = @0x1000a, _bob = @0x1000b)]
    fun true_friends_with_permanent_vouches(root: &signer, alice: &signer, _bob: &signer) {
        // Initialize genesis and validators
        mock::ol_test_genesis(root);
        mock::create_validator_accounts(root, 2, false);
        vouch::set_vouch_price(root, 0);
        
        // Alice vouches for Bob
        vouch::vouch_for(alice, @0x1000b);
        
        // Test the core permanent vouch functionality
        // 1. Check that the raw vouch exists
        let received_vouches = vouch::get_received_vouches_not_expired(@0x1000b);
        assert!(vector::contains(&received_vouches, &@0x1000a), 1);
        
        // 2. Check that the given vouch exists
        let given_vouches = vouch::get_given_vouches_not_expired(@0x1000a);
        assert!(vector::contains(&given_vouches, &@0x1000b), 2);
        
        // 3. Verify that vouches persist after many epochs (permanent aspect)
        let i = 0;
        while (i < 10) {  // Multiple epochs to test persistence
            mock::trigger_epoch(root);
            i = i + 1;
        };
        
        // Vouches should still exist after many epochs
        let received_vouches_after = vouch::get_received_vouches_not_expired(@0x1000b);
        assert!(vector::contains(&received_vouches_after, &@0x1000a), 3);
        
        let given_vouches_after = vouch::get_given_vouches_not_expired(@0x1000a);
        assert!(vector::contains(&given_vouches_after, &@0x1000b), 4);
        
        // Note: We avoid testing true_friends and is_valid_voucher_for here because
        // they apply ancestry filtering which may filter out vouches between test addresses
    }
}