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
    use ol_framework::gas_coin::GasCoin;
    use ol_framework::pledge_accounts;
    use diem_framework::coin;
    use diem_framework::transaction_fee;
    // use DiemFramework::Debug::print;

    /// for use on genesis, creates the infra escrow pledge policy struct
    public fun initialize_infra_pledge(vm: &signer) {
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

    // Transaction script for user to pledge to infra escrow.
    public fun user_pledge_infra(user_sig: &signer, amount: u64){

      pledge_accounts::user_pledge(user_sig, @ol_framework, amount);
    }

}