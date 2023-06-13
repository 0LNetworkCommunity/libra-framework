spec aptos_framework::transaction_fee {
    spec module {
        use aptos_framework::chain_status;
        pragma verify = true;
        pragma aborts_if_is_strict;

        invariant [suspendable] chain_status::is_operating() ==> exists<GasCoinCapabilities>(@aptos_framework);
    }

    spec CollectedFeesPerBlock {
        invariant burn_percentage <= 100;
    }

    spec initialize_fee_collection_and_distribution(aptos_framework: &signer, burn_percentage: u8) {
        // TODO: monomorphization issue. duplicated boogie procedures.
        pragma verify=false;
    }

    spec upgrade_burn_percentage(aptos_framework: &signer, new_burn_percentage: u8) {
        // TODO: missing aborts_if spec
        pragma verify=false;
    }

    spec register_proposer_for_fee_collection(proposer_addr: address) {
        aborts_if false;
        ensures is_fees_collection_enabled() ==>
            option::spec_borrow(global<CollectedFeesPerBlock>(@aptos_framework).proposer) == proposer_addr;
    }

    spec burn_coin_fraction(coin: &mut Coin<GasCoin>, burn_percentage: u8) {
        use aptos_framework::optional_aggregator;
        use aptos_framework::aggregator;
        use aptos_framework::coin::CoinInfo;
        use ol_framework::gas_coin::GasCoin;
        requires burn_percentage <= 100;
        requires exists<GasCoinCapabilities>(@aptos_framework);
        requires exists<CoinInfo<GasCoin>>(@aptos_framework);
        let amount_to_burn = (burn_percentage * coin::value(coin)) / 100;
        let maybe_supply = coin::get_coin_supply_opt<GasCoin>();
        aborts_if amount_to_burn > 0 && option::is_some(maybe_supply) && optional_aggregator::is_parallelizable(option::borrow(maybe_supply))
            && aggregator::spec_aggregator_get_val(option::borrow(option::borrow(maybe_supply).aggregator)) <
            amount_to_burn;
        aborts_if option::is_some(maybe_supply) && !optional_aggregator::is_parallelizable(option::borrow(maybe_supply))
            && option::borrow(option::borrow(maybe_supply).integer).value <
            amount_to_burn;
        include (amount_to_burn > 0) ==> coin::AbortsIfNotExistCoinInfo<GasCoin>;
    }

    spec fun collectedFeesAggregator(): AggregatableCoin<GasCoin> {
        global<CollectedFeesPerBlock>(@aptos_framework).amount
    }

    spec schema RequiresCollectedFeesPerValueLeqBlockAptosSupply {
        use aptos_framework::optional_aggregator;
        use aptos_framework::aggregator;
        let maybe_supply = coin::get_coin_supply_opt<GasCoin>();
        requires
            (is_fees_collection_enabled() && option::is_some(maybe_supply)) ==>
                (aggregator::spec_aggregator_get_val(global<CollectedFeesPerBlock>(@aptos_framework).amount.value) <=
                    optional_aggregator::optional_aggregator_value(option::spec_borrow(coin::get_coin_supply_opt<GasCoin>())));
    }

    spec process_collected_fees() {
        use aptos_framework::coin::CoinInfo;
        use ol_framework::gas_coin::GasCoin;
        requires exists<GasCoinCapabilities>(@aptos_framework);
        // requires exists<stake::ValidatorFees>(@aptos_framework);
        requires exists<CoinInfo<GasCoin>>(@aptos_framework);
        include RequiresCollectedFeesPerValueLeqBlockAptosSupply;
    }

    /// `GasCoinCapabilities` should be exists.
    spec burn_fee(account: address, fee: u64) {
        // TODO: complex aborts conditions in `burn_from`
        pragma aborts_if_is_partial;
        aborts_if !exists<GasCoinCapabilities>(@aptos_framework);
    }

    spec collect_fee(account: address, fee: u64) {
        use aptos_framework::aggregator;
        let collected_fees = global<CollectedFeesPerBlock>(@aptos_framework).amount;
        let aggr = collected_fees.value;
        aborts_if !exists<CollectedFeesPerBlock>(@aptos_framework);
        aborts_if fee > 0 && !exists<coin::CoinStore<GasCoin>>(account);
        aborts_if fee > 0 && global<coin::CoinStore<GasCoin>>(account).coin.value < fee;
        aborts_if fee > 0 && aggregator::spec_aggregator_get_val(aggr)
            + fee > aggregator::spec_get_limit(aggr);
        aborts_if fee > 0 && aggregator::spec_aggregator_get_val(aggr)
            + fee > MAX_U128;
    }

    /// Ensure caller is admin.
    /// Aborts if `GasCoinCapabilities` already exists.
    spec store_aptos_coin_burn_cap(aptos_framework: &signer, burn_cap: BurnCapability<GasCoin>) {
        use std::signer;
        let addr = signer::address_of(aptos_framework);
        aborts_if !system_addresses::is_aptos_framework_address(addr);
        aborts_if exists<GasCoinCapabilities>(addr);
        ensures exists<GasCoinCapabilities>(addr);
    }
}
