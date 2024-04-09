module ol_framework::musical_chairs {
    use diem_framework::chain_status;
    use diem_framework::system_addresses;
    use diem_framework::stake;
    use ol_framework::grade;
    use ol_framework::testnet;
    use std::fixed_point32;
    use std::vector;

    // use diem_std::debug::print;

    friend diem_framework::genesis;
    friend diem_framework::diem_governance;
    friend ol_framework::epoch_boundary;
    #[test_only]
    friend ol_framework::mock;


  /// we don't want to play the validator selection games
  /// before we're clear out of genesis
  const EPOCH_TO_START_EVAL: u64 = 2;

  /// we can't evaluate the performance of validators
  /// when there are too few rounds committed
  const MINIMUM_ROUNDS_PER_EPOCH: u64 = 1000;


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
    public(friend) fun initialize(
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
        epoch: u64,
        round: u64,
    ): (vector<address>, u64) acquires Chairs {
        system_addresses::assert_ol(vm);

        let validators = stake::get_current_validators();

        let (compliant_vals, _non, fail_ratio) =
        eval_compliance_impl(validators, epoch, round);

        let chairs = borrow_global_mut<Chairs>(@ol_framework);

        let num_compliant_nodes = vector::length(&compliant_vals);


        // Error handle. We should not have gone into an epoch where we
        // had MORE validators than seats offered.
        // If this happens it's because we are in some kind of a fork condition.
        // return with no changes
        if (num_compliant_nodes > chairs.seats_offered) {
          return (compliant_vals, chairs.seats_offered)
        };

        // The happiest case. All filled seats performed well in the last epoch
        if (fixed_point32::is_zero(*&fail_ratio)) { // handle this here to prevent multiplication error below
          chairs.seats_offered = chairs.seats_offered + 1;
          return (compliant_vals, chairs.seats_offered)
        };

        // Conditions under which seats should be one more than the number of compliant nodes(<= 5%)
        // Sad case. If we are not getting compliance, need to ratchet down the offer of seats in next epoch.
        // See below find_safe_set_size, how we determine what that number
        // should be
        let non_compliance_pct = fixed_point32::multiply_u64(100, *&fail_ratio);

        if (non_compliance_pct > 5) {
          // If network is bootstrapping don't reduce the seat count below
          // compliant nodes,

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
    public(friend) fun set_current_seats(framework: &signer, filled_seats: u64): u64 acquires Chairs{
      system_addresses::assert_diem_framework(framework);
      let chairs = borrow_global_mut<Chairs>(@ol_framework);
      chairs.seats_offered = filled_seats;
      chairs.seats_offered
    }

    // use the Case statistic to determine what proportion of the network is compliant.
    // private function prevent list DoS.
    fun eval_compliance_impl(
      validators: vector<address>,
      epoch: u64,
      _round: u64,
    ): (vector<address>, vector<address>, fixed_point32::FixedPoint32) {

        let val_set_len = vector::length(&validators);

        // skip first epoch, don't evaluate if the network doesn't have sufficient data
        if (epoch < EPOCH_TO_START_EVAL) return (validators, vector::empty<address>(), fixed_point32::create_from_rational(1, 1));

        let (compliant_nodes, non_compliant_nodes) = grade::eval_compliant_vals(validators);

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

    // Check for genesis, upgrade or recovery mode scenarios
    // if we are at genesis or otherwise at start of an epoch and don't
    // have a sufficient amount of history to evaluate nodes
    // we might reduce the validator set too agressively.
    // Musical chairs should not evaluate performance with less than 1000 rounds
    // created on mainnet,
    // there's something else very wrong in that case.

    fun is_booting_up(epoch: u64, round: u64): bool {
      !testnet::is_testnet() &&
      (epoch < EPOCH_TO_START_EVAL ||
      round < MINIMUM_ROUNDS_PER_EPOCH)
    }

    //////// GETTERS ////////

    #[view]
    public fun get_current_seats(): u64 acquires Chairs {
        borrow_global<Chairs>(@ol_framework).seats_offered
    }

    //////// TEST HELPERS ////////

    #[test_only]
    use diem_framework::chain_id;

    #[test_only]
    public fun test_stop(vm: &signer, epoch: u64, epoch_round: u64): (vector<address>, u64) acquires Chairs {
      stop_the_music(vm, epoch, epoch_round)
    }

    #[test_only]
    public fun test_eval_compliance(root: &signer, validators: vector<address>,
    epoch: u64, round: u64): (vector<address>, vector<address>,
    fixed_point32::FixedPoint32) {
      system_addresses::assert_ol(root);
      eval_compliance_impl(validators, epoch, round)

    }
    //////// TESTS ////////

    #[test(vm = @ol_framework)]
    public entry fun initialize_chairs(vm: signer) acquires Chairs {
      chain_id::initialize_for_test(&vm, 4);
      initialize(&vm, 10);
      assert!(get_current_seats() == 10, 1004);
    }
}
