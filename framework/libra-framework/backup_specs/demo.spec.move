spec ol_framework::demo {
    spec module {
        pragma verify = true;
    }

    spec print_this(account: &signer) {
      // should not abort
      pragma aborts_if_is_strict;
    }

    // TODO in strict mode
    spec set_message(account: &signer, message: string::String) {
      // pragma aborts_if_is_strict;
      // let events = borrow_global<MessageHolder>(signer::address_of(account)).message_change_events;
      // aborts_if event::counter(events) > MAX_U64;
    }

    spec get_message(addr: address): string::String {
      pragma aborts_if_is_strict;
      aborts_if !exists<MessageHolder>(addr);

    }
}
