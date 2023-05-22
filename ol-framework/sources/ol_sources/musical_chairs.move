module ol_framework::musical_chairs {
    use aptos_framework::chain_status;
    use aptos_framework::system_addresses;
    use aptos_framework::stake;
    use ol_framework::cases;
    // use ol_framework::globals;
    use std::fixed_point32;
    use std::vector;

    // use aptos_std::debug::print;

    struct Chairs has key {
        // The number of chairs in the game
        current_seats: u64,
        // A small history, for future use.
        history: vector<u64>,
    }

    // With musical chairs we are trying to estimate
    // the number of nodes which the network can support
    // BFT has upperbounds in the low hundreds, but we
    // don't need to hard code it.
    // There also needs to be an upper bound so that there is some
    // competition among validators.
    // Instead of hard coding a number, and needing to reach social
    // consensus to change it:  we'll determine the size based on
    // the network's performance as a whole.
    // There are many metrics that can be used. For now we'll use
    // a simple heuristic that is already on chain: compliant node cardinality.
    // Other heuristics may be explored, so long as the information
    // reliably committed to the chain.

    // The rules:
    // Validators who perform, are not guaranteed entry into the
    // next epoch of the chain. All we are establishing is the ceiling
    // for validators.
    // When the 100% of the validators are performing well
    // the network can safely increase the threshold by 1 node.
    // We can also say if less that 5% fail, no change happens.
    // When the network is performing poorly, greater than 5%,
    // the threshold is reduced not by a predetermined unit, but
    // to the number of compliant and performant nodes.

    /// Called by root in genesis to initialize the GAS coin
    public fun initialize(
        vm: &signer,
        genesis_seats: u64,
    ) {
        // system_addresses::assert_vm(vm);
        // TODO: replace with VM
        system_addresses::assert_ol(vm);

        chain_status::is_genesis();
        if (exists<Chairs>(@ol_framework)) {
            return
        };

        move_to(vm, Chairs {
            current_seats: genesis_seats,
            history: vector::empty<u64>(),
        });
    }

    // get the number of seats in the game
    // TODO: make this a (friend)
    public fun stop_the_music( // sorry, had to.
      vm: &signer,
      // height_start: u64,
      // height_end: u64
    ): (vector<address>, u64) acquires Chairs {
        system_addresses::assert_ol(vm);

        let validators = stake::get_current_validators();


        let (compliant, _non, ratio) = eval_compliance_impl(validators);



        let chairs = borrow_global_mut<Chairs>(@ol_framework);

        // if the ratio of non-compliant nodes is between 0 and 5%
        // we can increase the number of chairs by 1.
        // otherwise (if more than 5% are failing) we wall back to the size of ther performant set.
        if (fixed_point32::is_zero(*&ratio)) { // catch zeros
          chairs.current_seats = chairs.current_seats + 1;
        } else if (fixed_point32::multiply_u64(100, *&ratio) <= 5){
          chairs.current_seats = chairs.current_seats + 1;
        } else if (fixed_point32::multiply_u64(100, *&ratio) > 5) {
          // remove chairs
          // reduce the validator set to the size of the compliant set.
          chairs.current_seats = vector::length(&compliant);
        };
        // otherwise do nothing, the validator set is within a tolerable range.

        (compliant, chairs.current_seats)
    }

    #[test_only]
    public fun test_eval_compliance(root: &signer, validators: vector<address>): (vector<address>, vector<address>, fixed_point32::FixedPoint32) {
      system_addresses::assert_ol(root);
      eval_compliance_impl(validators)

    }
    // use the Case statistic to determine what proportion of the network is compliant.
    // private function prevent list DoS.
    fun eval_compliance_impl(
      validators: vector<address>,
    ) : (vector<address>, vector<address>, fixed_point32::FixedPoint32) {
        
        let val_set_len = vector::length(&validators);

        let compliant_nodes = vector::empty<address>();
        let non_compliant_nodes = vector::empty<address>();

        let i = 0;
        while (i < val_set_len) {
            let addr = *vector::borrow(&validators, i);
            if (cases::get_case(addr) == 1) {
                vector::push_back(&mut compliant_nodes, addr);
            } else {
                vector::push_back(&mut non_compliant_nodes, addr);
            };
            i = i + 1;
        };

        let good_len = vector::length(&compliant_nodes) ;
        let bad_len = vector::length(&non_compliant_nodes);

        // Note: sorry for repetition but necessary for writing tests and debugging.
        let null = fixed_point32::create_from_raw_value(0);
        if (good_len > val_set_len) { // safety
          return (vector::empty(), vector::empty(), null)
        };

        if (bad_len > val_set_len) { // safety
          return (vector::empty(), vector::empty(), null)
        };

        if ((good_len + bad_len) != val_set_len) { // safety
          return (vector::empty(), vector::empty(), null)
        };


        let ratio = if (bad_len > 0) {
          fixed_point32::create_from_rational(bad_len, val_set_len)
        } else {
          null
        };

        (compliant_nodes, non_compliant_nodes, ratio)
    }

    //////// GETTERS ////////

    public fun get_current_seats(): u64 acquires Chairs {
        borrow_global<Chairs>(@ol_framework).current_seats
    }

    #[test_only]
    use aptos_framework::chain_id;

    //////// TESTS ////////

    #[test(vm = @ol_framework)]
    public entry fun initialize_chairs(vm: signer) acquires Chairs {
      chain_id::initialize_for_test(&vm, 4);
      initialize(&vm, 10);
      assert!(get_current_seats() == 10, 1004);
    }
}
