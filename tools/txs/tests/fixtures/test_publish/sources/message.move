module this_address::message {
    // use std::string;
    use std::signer;
    // use aptos_std::debug;

    struct MessageHolder has key {
        message: u64,
    }

    #[view]
    public fun read(account: signer): u64 acquires MessageHolder {
      let account_addr = signer::address_of(&account);

      let state = borrow_global<MessageHolder>(account_addr);
      state.message
    }

    public entry fun set_message(account: signer, num: u64)
    acquires MessageHolder {
        let account_addr = signer::address_of(&account);
        if (!exists<MessageHolder>(account_addr)) {
            move_to(&account, MessageHolder {
                message: num
            })
        } else {
            let old_message_holder = borrow_global_mut<MessageHolder>(account_addr);
            old_message_holder.message = num;
        }
    }

}
