spec ol_framework::ol_account {
    spec module {
        pragma verify = true;
        // pragma aborts_if_is_strict;
        invariant [suspendable] chain_status::is_operating() ==> exists<gas_coin::FinalMint>(@diem_framework);
        invariant [suspendable] chain_status::is_operating() ==>  exists<coin::CoinInfo<GasCoin>>(@diem_framework);

    }

    // /// Check if the bytes of the auth_key is 32.
    // /// The Account does not exist under the auth_key before creating the account.
    // /// Limit the address of auth_key is not @vm_reserved / @diem_framework / @diem_toke.
    // spec create_account(auth_key: address) {
    //     include CreateAccountAbortsIf;
    //     ensures exists<account::Account>(auth_key);
    //     ensures exists<coin::CoinStore<GasCoin>>(auth_key);
    // }
    spec schema CreateAccountAbortsIf {
        auth_key: address;
        aborts_if exists<account::Account>(auth_key);
        aborts_if length_judgment(auth_key);
        aborts_if auth_key == @vm_reserved || auth_key == @diem_framework || auth_key == @diem_token;
    }

    spec fun length_judgment(auth_key: address): bool {
        use std::bcs;

        let authentication_key = bcs::to_bytes(auth_key);
        len(authentication_key) != 32
    }

    // ol_account::withdraw can never use more than the slow wallet limit
    spec withdraw(sender: &signer, amount: u64): Coin<GasCoin>{
        include AssumeCoinRegistered;

        let account_addr = signer::address_of(sender);
        aborts_if amount == 0;

        let coin_store = global<coin::CoinStore<GasCoin>>(account_addr);
        let balance = coin_store.coin.value;

        aborts_if balance < amount;

        // in the case of slow wallets
        let slow_store = global<slow_wallet::SlowWallet>(account_addr);
        aborts_if exists<slow_wallet::SlowWallet>(account_addr) &&
        slow_store.unlocked < amount;

        ensures result == Coin<GasCoin>{value: amount};
    }

    spec schema AssumeCoinRegistered {
        sender: &signer;
        let account_addr = signer::address_of(sender);
        aborts_if !coin::is_account_registered<GasCoin>(account_addr);
    }

    spec assert_account_exists(addr: address) {
        aborts_if !account::exists_at(addr);
    }

    /// Check if the address existed.
    /// Check if the GasCoin under the address existed.
    spec assert_account_is_registered_for_gas(addr: address) {
        aborts_if !account::exists_at(addr);
        aborts_if !coin::is_account_registered<GasCoin>(addr);
    }

    spec set_allow_direct_coin_transfers(account: &signer, allow: bool) {
        let addr = signer::address_of(account);
        include !exists<DirectTransferConfig>(addr) ==> account::NewEventHandleAbortsIf;
    }

    // spec batch_transfer(source: &signer, recipients: vector<address>, amounts: vector<u64>) {
    //     // TODO: missing aborts_if spec
    //     pragma verify=false;
    // }

    spec can_receive_direct_coin_transfers(account: address): bool {
        aborts_if false;
        ensures result == (
            !exists<DirectTransferConfig>(account) ||
                global<DirectTransferConfig>(account).allow_arbitrary_coin_transfers
        );
    }

    // spec batch_transfer_coins<CoinType>(from: &signer, recipients: vector<address>, amounts: vector<u64>) {
    //     // TODO: missing aborts_if spec
    //     pragma verify=false;
    // }

    // spec deposit_coins<CoinType>(to: address, coins: Coin<CoinType>) {
    //     // TODO: missing aborts_if spec
    //     pragma verify=false;
    // }

    // spec transfer_coins<CoinType>(from: &signer, to: address, amount: u64) {
    //     // TODO: missing aborts_if spec
    //     pragma verify=false;
    // }
}
