spec aptos_framework::optional_aggregator {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec OptionalAggregator {
        invariant option::is_some(aggregator) <==> option::is_none(integer);
        invariant option::is_some(integer) <==> option::is_none(aggregator);
        invariant option::is_some(integer) ==> option::borrow(integer).value <= option::borrow(integer).limit;
        invariant option::is_some(aggregator) ==> aggregator::spec_aggregator_get_val(option::borrow(aggregator)) <=
            aggregator::spec_get_limit(option::borrow(aggregator));
    }

    spec new_integer(limit: u128): Integer {
        aborts_if false;
        ensures result.limit == limit;
        ensures result.value == 0;
    }

    /// Check for overflow.
    spec add_integer(integer: &mut Integer, value: u128) {
        aborts_if value > (integer.limit - integer.value);
        aborts_if integer.value + value > MAX_U128;
        ensures integer.value <= integer.limit;
        ensures integer.value == old(integer.value) + value;
    }

    spec sub(optional_aggregator: &mut OptionalAggregator, value: u128) {
        aborts_if is_parallelizable(optional_aggregator) && (aggregator::spec_aggregator_get_val(option::borrow(optional_aggregator.aggregator))
            < value);
        aborts_if !is_parallelizable(optional_aggregator) &&
            (option::borrow(optional_aggregator.integer).value < value);
        ensures ((optional_aggregator_value(optional_aggregator) == optional_aggregator_value(old(optional_aggregator)) - value));
    }

    spec read(optional_aggregator: &OptionalAggregator): u128 {
        ensures !is_parallelizable(optional_aggregator) ==> result == option::borrow(optional_aggregator.integer).value;
        ensures is_parallelizable(optional_aggregator) ==>
            result == aggregator::spec_read(option::borrow(optional_aggregator.aggregator));
    }

    spec add(optional_aggregator: &mut OptionalAggregator, value: u128) {
        aborts_if is_parallelizable(optional_aggregator) && (aggregator::spec_aggregator_get_val(option::borrow(optional_aggregator.aggregator))
            + value > aggregator::spec_get_limit(option::borrow(optional_aggregator.aggregator)));
        aborts_if is_parallelizable(optional_aggregator) && (aggregator::spec_aggregator_get_val(option::borrow(optional_aggregator.aggregator))
            + value > MAX_U128);
        aborts_if !is_parallelizable(optional_aggregator) &&
            (option::borrow(optional_aggregator.integer).value + value > MAX_U128);
        aborts_if !is_parallelizable(optional_aggregator) &&
            (value > (option::borrow(optional_aggregator.integer).limit - option::borrow(optional_aggregator.integer).value));
        ensures ((optional_aggregator_value(optional_aggregator) == optional_aggregator_value(old(optional_aggregator)) + value));
    }

    spec switch(optional_aggregator: &mut OptionalAggregator) {
        let vec_ref = optional_aggregator.integer.vec;
        aborts_if is_parallelizable(optional_aggregator) && len(vec_ref) != 0;
        aborts_if !is_parallelizable(optional_aggregator) && len(vec_ref) == 0;
        aborts_if !is_parallelizable(optional_aggregator) && !exists<aggregator_factory::AggregatorFactory>(@aptos_framework);
        ensures optional_aggregator_value(optional_aggregator) == optional_aggregator_value(old(optional_aggregator));
    }

    spec sub_integer(integer: &mut Integer, value: u128) {
        aborts_if value > integer.value;
        ensures integer.value == old(integer.value) - value;
    }

    spec new(limit: u128, parallelizable: bool): OptionalAggregator {
        aborts_if parallelizable && !exists<aggregator_factory::AggregatorFactory>(@aptos_framework);
        ensures parallelizable ==> is_parallelizable(result);
        ensures !parallelizable ==> !is_parallelizable(result);
        ensures optional_aggregator_value(result) == 0;
        ensures optional_aggregator_value(result) <= optional_aggregator_limit(result);
    }

    /// Option<Integer> does not exist When Option<Aggregator> exists.
    /// Option<Integer> exists when Option<Aggregator> does not exist.
    /// The AggregatorFactory is under the @aptos_framework when Option<Aggregator> does not exist.
    spec switch_and_zero_out(optional_aggregator: &mut OptionalAggregator) {
        let vec_ref = optional_aggregator.integer.vec;
        aborts_if is_parallelizable(optional_aggregator) && len(vec_ref) != 0;
        aborts_if !is_parallelizable(optional_aggregator) && len(vec_ref) == 0;
        aborts_if !is_parallelizable(optional_aggregator) && !exists<aggregator_factory::AggregatorFactory>(@aptos_framework);
        ensures is_parallelizable(old(optional_aggregator)) ==> !is_parallelizable(optional_aggregator);
        ensures !is_parallelizable(old(optional_aggregator)) ==> is_parallelizable(optional_aggregator);
        ensures optional_aggregator_value(optional_aggregator) == 0;
    }

    /// The aggregator exists and the integer dosex not exist when Switches from parallelizable to non-parallelizable implementation.
    spec switch_to_integer_and_zero_out(
        optional_aggregator: &mut OptionalAggregator
    ): u128 {
        let limit = aggregator::spec_get_limit(option::borrow(optional_aggregator.aggregator));
        aborts_if len(optional_aggregator.aggregator.vec) == 0;
        aborts_if len(optional_aggregator.integer.vec) != 0;
        ensures !is_parallelizable(optional_aggregator);
        ensures option::borrow(optional_aggregator.integer).limit == limit;
        ensures option::borrow(optional_aggregator.integer).value == 0;
    }

    /// The integer exists and the aggregator does not exist when Switches from non-parallelizable to parallelizable implementation.
    /// The AggregatorFactory is under the @aptos_framework.
    spec switch_to_aggregator_and_zero_out(
        optional_aggregator: &mut OptionalAggregator
    ): u128 {
        let limit = option::borrow(optional_aggregator.integer).limit;
        aborts_if len(optional_aggregator.integer.vec) == 0;
        aborts_if !exists<aggregator_factory::AggregatorFactory>(@aptos_framework);
        aborts_if len(optional_aggregator.aggregator.vec) != 0;
        ensures is_parallelizable(optional_aggregator);
        ensures aggregator::spec_get_limit(option::borrow(optional_aggregator.aggregator)) == limit;
        ensures aggregator::spec_aggregator_get_val(option::borrow(optional_aggregator.aggregator)) == 0;
    }

    spec destroy(optional_aggregator: OptionalAggregator) {
        aborts_if is_parallelizable(optional_aggregator) && len(optional_aggregator.integer.vec) != 0;
        aborts_if !is_parallelizable(optional_aggregator) && len(optional_aggregator.integer.vec) == 0;
    }

    /// The aggregator exists and the integer does not exist when destroy the aggregator.
    spec destroy_optional_aggregator(optional_aggregator: OptionalAggregator): u128 {
        aborts_if len(optional_aggregator.aggregator.vec) == 0;
        aborts_if len(optional_aggregator.integer.vec) != 0;
        ensures result == aggregator::spec_get_limit(option::borrow(optional_aggregator.aggregator));
    }

    /// The integer exists and the aggregator does not exist when destroy the integer.
    spec destroy_optional_integer(optional_aggregator: OptionalAggregator): u128 {
        aborts_if len(optional_aggregator.integer.vec) == 0;
        aborts_if len(optional_aggregator.aggregator.vec) != 0;
        ensures result == option::borrow(optional_aggregator.integer).limit;
    }

    spec fun optional_aggregator_value(optional_aggregator: OptionalAggregator): u128 {
        if (is_parallelizable(optional_aggregator)) {
            aggregator::spec_aggregator_get_val(option::borrow(optional_aggregator.aggregator))
        } else {
            option::borrow(optional_aggregator.integer).value
        }
    }

    spec fun optional_aggregator_limit(optional_aggregator: OptionalAggregator): u128 {
        if (is_parallelizable(optional_aggregator)) {
            aggregator::spec_get_limit(option::borrow(optional_aggregator.aggregator))
        } else {
            option::borrow(optional_aggregator.integer).limit
        }
    }

}
