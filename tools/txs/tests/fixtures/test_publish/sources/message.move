module this_address::message {
    use std::signer;

    struct MessageHolder has key {
        message: u64,
    }

    #[view]
    public fun read(account: address): u64 acquires MessageHolder {
      let state = borrow_global<MessageHolder>(account);
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
