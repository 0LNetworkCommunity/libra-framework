module ol_framework::musical_chairs {
    use diem_framework::chain_status;
    use diem_framework::system_addresses;
    use diem_framework::stake;
    use ol_framework::grade;
    use std::fixed_point32;
    use std::vector;
    // use diem_std::debug::print;

    friend ol_framework::epoch_boundary;

    struct Chairs has key {
        // The number of chairs in the game
        seats_offered: u64,
        // TODO: A small history, for future use.
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
            seats_offered: genesis_seats,
            history: vector::empty<u64>(),
        });
    }

    /// get the number of seats in the game
    /// returns the list of compliant validators and the number of seats
    /// we should offer in the next epoch
    /// (compliant_vals, seats_offered)
    public(friend) fun stop_the_music( // sorry, had to.
        vm: &signer,
        epoch_round: u64,
    ): (vector<address>, u64) acquires Chairs {
        system_addresses::assert_ol(vm);

        let validators = stake::get_current_validators();
        let (compliant_vals, _non, ratio) = eval_compliance_impl(validators, epoch_round);

        let chairs = borrow_global_mut<Chairs>(@ol_framework);

        let num_compliant_nodes = vector::length(&compliant_vals);

        // failover, there should not be more compliant nodes than seats that
        // were offered.

        // return with no changes
        if (num_compliant_nodes > chairs.seats_offered) {
          return (compliant_vals, chairs.seats_offered)
        };

        // The happiest case. All filled seats performed well in the last epoch
        if (fixed_point32::is_zero(*&ratio)) { // handle this here to prevent multiplication error below
          chairs.seats_offered = chairs.seats_offered + 1;
          return (compliant_vals, chairs.seats_offered)
        };


        let non_compliance_pct = fixed_point32::multiply_u64(100, *&ratio);

        // Conditions under which seats should be one more than the number of compliant nodes(<= 5%)
        // Sad case. If we are not getting compliance, need to ratchet down the offer of seats in next epoch.
        // See below find_safe_set_size, how we determine what that number should be
        if (non_compliance_pct > 5) {
            chairs.seats_offered = num_compliant_nodes;
        } else {
            // Ok case. If it's between 0 and 5% then we accept that margin as if it was fully compliant
            chairs.seats_offered = chairs.seats_offered + 1;
        };

        // catch failure mode
        // mostly for genesis, or testnets
        if (chairs.seats_offered < 4) {
          chairs.seats_offered = 4;
        };

        (compliant_vals, chairs.seats_offered)
    }

    // Update seat count to match filled seats post-PoF auction.
    // in case we were not able to fill all the seats offered
    // we don't want to keep incrementing from a baseline which we cannot fill
    // it can spiral out of range.
    public fun set_current_seats(vm: &signer, filled_seats: u64): u64 acquires Chairs{
      system_addresses::assert_ol(vm);
      let chairs = borrow_global_mut<Chairs>(@ol_framework);
      chairs.seats_offered = filled_seats;
      chairs.seats_offered
    }

    #[test_only]
    public fun test_eval_compliance(root: &signer, validators: vector<address>, epoch_round: u64): (vector<address>, vector<address>, fixed_point32::FixedPoint32) {
      system_addresses::assert_ol(root);
      eval_compliance_impl(validators, epoch_round)

    }
    // use the Case statistic to determine what proportion of the network is compliant.
    // private function prevent list DoS.
    fun eval_compliance_impl(
      validators: vector<address>,
      epoch: u64,
    ) : (vector<address>, vector<address>, fixed_point32::FixedPoint32) {

        let val_set_len = vector::length(&validators);

        let compliant_nodes = vector::empty<address>();
        let non_compliant_nodes = vector::empty<address>();

        // if we are at genesis or otherwise at start of an epoch, we don't
        // want to brick the validator set
        // TODO: use status.move is_operating
        if (epoch < 2) return (validators, non_compliant_nodes, fixed_point32::create_from_rational(1, 1));

        let (highest_net_props, _val) = stake::get_highest_net_proposer();
        let i = 0;
        while (i < val_set_len) {
            let addr = *vector::borrow(&validators, i);
            let (compliant, _, _, _) = grade::get_validator_grade(addr, highest_net_props);
            // let compliant = true;
            if (compliant) {
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
        borrow_global<Chairs>(@ol_framework).seats_offered
    }

    #[test_only]
    use diem_framework::chain_id;

    #[test_only]
    public fun test_stop(vm: &signer, epoch_round: u64): (vector<address>, u64) acquires Chairs {
      stop_the_music(vm, epoch_round)
    }

    //////// TESTS ////////

    #[test(vm = @ol_framework)]
    public entry fun initialize_chairs(vm: signer) acquires Chairs {
      chain_id::initialize_for_test(&vm, 4);
      initialize(&vm, 10);
      assert!(get_current_seats() == 10, 1004);
    }
}
