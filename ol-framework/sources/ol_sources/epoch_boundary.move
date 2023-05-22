
module ol_framework::epoch_boundary {

    use std::signer;
    use ol_framework::slow_wallet;
    use ol_framework::musical_chairs;
    use ol_framework::proof_of_fee;

    friend aptos_framework::block;

    // Contains all of 0L's business logic for end of epoch.
    // This removed business logic from reconfiguration.move
    // and prevents dependency cycling.
    public(friend) fun epoch_boundary(root: &signer) {
        if (signer::address_of(root) != @ol_framework) {
            return
        };

        // TODO: this needs to be a friend function, but it's in a different namespace, so we are gating it with vm signer, which is what was done previously. Which means hacking block.move
        slow_wallet::on_new_epoch(root);
        let (compliant, n_seats) = musical_chairs::stop_the_music(root);
        proof_of_fee::end_epoch(root, &compliant, n_seats);

    }


    #[test_only]
    public fun ol_reconfigure_for_test(vm: &signer) {
        use aptos_framework::system_addresses;

        system_addresses::assert_ol(vm);
        epoch_boundary(vm);
    }

}