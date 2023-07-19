
module ol_framework::fee_maker {

    use ol_framework::system_addresses;
    use std::vector;

    /// FeeMaker struct lives on an individual's account
    /// We check how many fees the user has paid.
    /// This will interact with Burn preferences when there is a remainder of fees in the TransactionFee account
    struct FeeMaker has key {
      epoch: u64,
      lifetime: u64,
    }

    /// We need a list of who is producing fees this epoch.
    /// This lives on the VM address
    struct EpochFeeMakerRegistry has key {
      fee_makers: vector<address>,
      epoch_fees_made: u64,
    }

    /// Initialize the registry at the VM address.
    public fun initialize(vm: &signer) {
      system_addresses::assert_ol(vm);
      let registry = EpochFeeMakerRegistry {
        fee_makers: vector::empty(),
        epoch_fees_made: 0,
      };
      move_to(vm, registry);
    }

    //TODO: very few accounts will need this, perhaps lazily initialize?
    /// FeeMaker is initialized when the account is created
    public fun initialize_fee_maker(account: &signer) {
      let fee_maker = FeeMaker {
        epoch: 0,
        lifetime: 0,
      };
      move_to(account, fee_maker);
    }

    public fun epoch_reset_fee_maker(vm: &signer) acquires EpochFeeMakerRegistry, FeeMaker {
      system_addresses::assert_ol(vm);
      let registry = borrow_global_mut<EpochFeeMakerRegistry>(@ol_framework);
      let fee_makers = &registry.fee_makers;

      let i = 0;
      while (i < vector::length(fee_makers)) {
        let account = *vector::borrow(fee_makers, i);
        reset_one_fee_maker(vm, account);
        i = i + 1;
      };
      registry.fee_makers = vector::empty();
      registry.epoch_fees_made = 0;
    }

    /// FeeMaker is reset at the epoch boundary, and the lifetime is updated.
    fun reset_one_fee_maker(vm: &signer, account: address) acquires FeeMaker {
      system_addresses::assert_vm(vm);
      let fee_maker = borrow_global_mut<FeeMaker>(account);
        fee_maker.lifetime = fee_maker.lifetime + fee_maker.epoch;
        fee_maker.epoch = 0;
    }

    /// add a fee to the account fee maker for an epoch
    /// PRIVATE function
    fun track_user_fee(account: address, amount: u64) acquires FeeMaker, EpochFeeMakerRegistry {
      if (!exists<FeeMaker>(account)) {
        return
      };

      let fee_maker = borrow_global_mut<FeeMaker>(account);
      fee_maker.epoch = fee_maker.epoch + amount;

      // update the registry
      let registry = borrow_global_mut<EpochFeeMakerRegistry>(@ol_framework);
      if (!vector::contains(&registry.fee_makers, &account)) {
        vector::push_back(&mut registry.fee_makers, account);
      };
      registry.epoch_fees_made = registry.epoch_fees_made + amount;
    }

    //////// GETTERS ///////

    // get list of fee makers
    public fun get_fee_makers(): vector<address> acquires EpochFeeMakerRegistry {
      let registry = borrow_global<EpochFeeMakerRegistry>(@ol_framework);
      *&registry.fee_makers
    }

    #[view]
    /// get the fees made by the user in the epoch
    public fun get_user_fees_made(account: address): u64 acquires FeeMaker {
      if (!exists<FeeMaker>(account)) {
        return 0
      };
      let fee_maker = borrow_global<FeeMaker>(account);
      fee_maker.epoch
    }

    #[view]
    /// get total fees made across all epochs
    public fun get_all_fees_made(): u64 acquires EpochFeeMakerRegistry {
      if (!exists<EpochFeeMakerRegistry>(@ol_framework)) return 0;

      let registry = borrow_global<EpochFeeMakerRegistry>(@ol_framework);
      registry.epoch_fees_made
    }

}
