///////////////////////////////////////////////////////////////////////////
// 0L Module
// Infra Escrow
///////////////////////////////////////////////////////////////////////////
// Controls funds that have been pledged to infrastructure subsidy
// Like other Pledged segregated accounts, the value lives on the
// user's account. The funding is not pooled into a system account.
// According to the policy the funds may be drawn down from Pledged
// segregated accounts.
///////////////////////////////////////////////////////////////////////////

module ol_framework::infra_escrow{
    use std::option::{Self, Option};
    use aptos_framework::system_addresses;
    use ol_framework::gas_coin::GasCoin;
    use ol_framework::pledge_accounts;
    use ol_framework::slow_wallet;
    use aptos_framework::coin;
    use aptos_framework::transaction_fee;
    use std::fixed_point32;
    use std::signer;
    // use aptos_std::debug::print;

    /// for use on genesis, creates the infra escrow pledge policy struct
    public fun initialize(vm: &signer) {
        system_addresses::assert_ol(vm);
        // TODO: perhaps this policy needs to be published to a different address?
        pledge_accounts::publish_beneficiary_policy(
          vm, // only VM calls at genesis
          b"infra escrow",
          90,
          true
        );
    }

    /// VM can call down pledged funds.
    public fun infra_pledge_withdraw(vm: &signer, amount: u64): Option<coin::Coin<GasCoin>> {
        system_addresses::assert_ol(vm);
        pledge_accounts::withdraw_from_all_pledge_accounts(vm, amount)
    }

    /// Helper for epoch boundaries.
    /// Collects funds from pledge and places temporarily in network account (TransactionFee account)
    public fun epoch_boundary_collection(root: &signer, amount: u64) {
        system_addresses::assert_ol(root);
        let opt = pledge_accounts::withdraw_from_all_pledge_accounts(root, amount);

        if (option::is_none(&opt)) {
          option::destroy_none(opt);
          return
        };
        let c = option::extract(&mut opt);
        option::destroy_none(opt);

        transaction_fee::pay_fee(root, c);
    }

    /// for an uprade using an escrow percent. Only to be called at genesis
    // escrow percent has 6 decimal precision (1m);
    public fun fork_escrow_init(vm: &signer, user_sig: &signer, escrow_pct: u64) {
        system_addresses::assert_ol(vm);
        let user_addr = signer::address_of(user_sig);
      // // establish the infrastructure escrow pledge
      // if (is_validator) {
        let escrow_pct = fixed_point32::create_from_rational(escrow_pct, 1000000);
      //   // TODO: get locked amount
      let (unlocked, total) = slow_wallet::balance(user_addr);

      let locked = 0;
      if ((total > unlocked) && (total > 0)) {
        locked = (total - unlocked);
      };

      if (locked > 0) {
        let to_escrow = fixed_point32::multiply_u64(locked, escrow_pct);
        // print(&to_escrow);
        pledge_accounts::genesis_infra_escrow_pledge(vm, user_sig, to_escrow)
      };
    }

    // Transaction script for user to pledge to infra escrow.
    public fun user_pledge_infra(user_sig: &signer, amount: u64){

      pledge_accounts::user_pledge(user_sig, @ol_framework, amount);
    }

    #[view]
    /// gets the amount a user has pledged to infra escrow
    public fun user_infra_pledge_balance(addr: address): u64 {
      pledge_accounts::get_user_pledge_amount(addr, @ol_framework)
    }

    #[view]
    /// gets the amount a user has pledged to infra escrow
    public fun infra_escrow_balance(): u64 {
      pledge_accounts::get_available_to_beneficiary(@ol_framework)
    }

}