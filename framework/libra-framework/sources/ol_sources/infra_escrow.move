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
    use diem_framework::system_addresses;
    use ol_framework::libra_coin::LibraCoin;
    use ol_framework::pledge_accounts;
    // use ol_framework::slow_wallet;
    use ol_framework::ol_account;
    use diem_framework::coin;
    use diem_framework::transaction_fee;
    // use std::fixed_point32;
    // use std::signer;
    use std::error;
    // use diem_std::debug::print;

    friend ol_framework::epoch_boundary;
    friend diem_framework::genesis;

    const EGENESIS_REWARD: u64 = 0;
    /// for use on genesis, creates the infra escrow pledge policy struct
    public fun initialize(vm: &signer) {
        // NOTE: THIS MUST BE THE 0x0 address, because on epoch boundary it is that address @vm_reserved which will be calling the functions.
        system_addresses::assert_vm(vm);
        // TODO: perhaps this policy needs to be published to a different address?
        pledge_accounts::publish_beneficiary_policy(
          vm, // only VM calls at genesis
          b"infra escrow",
          90,
          true
        );
    }

    /// VM can call down pledged funds.
    // NOTE: the signer MUST_BE 0x0 address
    fun infra_pledge_withdraw(vm: &signer, amount: u64): Option<coin::Coin<LibraCoin>> {
        system_addresses::assert_ol(vm);
        pledge_accounts::withdraw_from_all_pledge_accounts(vm, amount)
    }

    /// Helper for epoch boundaries.
    /// Collects funds from pledge and places temporarily in network account
    // (the TransactionFee account)
    /// @return tuple of 2
    /// 0: if collection succeeded
    /// 1: how much was collected
    public(friend) fun epoch_boundary_collection(root: &signer, amount: u64):
    (bool, u64) {
        system_addresses::assert_ol(root);
        let opt = pledge_accounts::withdraw_from_all_pledge_accounts(root, amount);

        if (option::is_none(&opt)) {
          option::destroy_none(opt);
          return (false, 0)
        };
        let c = option::extract(&mut opt);
        option::destroy_none(opt);
        let value = coin::value(&c);
        transaction_fee::vm_pay_fee(root, @vm_reserved, c); // don't attribute
        // to the user
        return(true, value)
    }



    // Transaction script for user to pledge to infra escrow.
    public fun user_pledge_infra(user_sig: &signer, amount: u64){

      pledge_accounts::user_pledge(user_sig, @vm_reserved, amount);
    }

    #[view]
    /// gets the amount a user has pledged to infra escrow
    public fun user_infra_pledge_balance(addr: address): u64 {
      pledge_accounts::get_user_pledge_amount(addr, @vm_reserved)
    }

    #[view]
    /// gets the amount a user has pledged to infra escrow
    public fun infra_escrow_balance(): u64 {
      pledge_accounts::get_available_to_beneficiary(@vm_reserved)
    }

    //////// TESTNET HELPERS ////////
    /// For testnet scenarios we may want to mint a minimal coin to the validators
    // this is only called through genesis when using the production rust libra-genesis-tool
    // and in the move code, we want the validators to start with zero balances
    // and add them with mock.move when we need it.
    public fun genesis_coin_validator(root: &signer, to: address) {
      system_addresses::assert_ol(root);
      let bootstrap_amount = 1000000000;
      if (infra_escrow_balance() > bootstrap_amount) {
        let c_opt = infra_pledge_withdraw(root, bootstrap_amount);
        assert!(option::is_some(&c_opt), error::invalid_state(EGENESIS_REWARD));
        // if (option::is_some(&c_opt)) {
          let coin = option::extract(&mut c_opt);
          ol_account::deposit_coins(to, coin);
        // };
        option::destroy_none(c_opt);
      }
    }
}
