
/////////////////////////////////////////////////////////////////////////
// 0L Module
// Demo Persistence
// Error Code: 0400
/////////////////////////////////////////////////////////////////////////

module ol_framework::persistence_demo {
    use std::vector;
    use std::signer;
    use std::error;
    use ol_framework::testnet::is_testnet;

    // In Move the types for data storage are `resource struct`. Here a type
    // State is being defined. Once a type is initialized in the global state,
    // the resource is treated as if in-memory on the heap, the diem database
    // is abstracted. The data is namespaced by an "access path" which includes
    // the module name and user address. No special APIs are necessary for reading
    // from the database, except permissioning each function which accesses a given
    // struct, more below.
    struct State has key {
      hist: vector<u8>,
    }

    /// this operation can only be performed on testnet
    const ETESTNET : u64 = 1;

    // For this demo, the `initialize` function writes a PersistenceDemo::State
    // resource at the "sender" address. The access path will be
    // <sender address>/PersistenceDemo/State/
    public fun initialize(sender: &signer){
      // `assert can be used to evaluate a bool and exit the program with
      // an error code, e.g. testing if this is being run in testnet, and
      // throwing error 01.
      assert!(is_testnet(), error::invalid_state(ETESTNET));
      // In the actual module, must assert that this is the sender is the association
      move_to<State>(sender, State{ hist: vector::empty() });
    }

    // A simple example/demo spec
    spec initialize {
        let addr = signer::address_of(sender);
        // Note: Change this to non-zero value to get move prover error
        let init_size = 0;
        ensures vector::length(global<State>(addr).hist) == init_size;
    }

    // To read or write to a Resource Struct an `acquires` tag is needed to
    // permission a function. NOTE all downsteam functions will also need
    // permission on that data struct, i.e. need the same `acquires` parameters.
    public fun add_stuff(sender: &signer) acquires State {
      assert!(is_testnet(), error::invalid_state(ETESTNET));

      // Resource Struct state is always "borrowed" and "moved" and generally
      // cannot be copied. A struct can be mutably borrowed, if it is written to,
      // using `borrow_global_mut`. Note the Type State
      let st = borrow_global_mut<State>(signer::address_of(sender));
      // the `&` as in Rust makes the assignment to a borrowed value. Each
      // vector operation below with use a st.hist and return it before the
      // next one can execute.
      let s = &mut st.hist;

      // Move has very limited data types. vector is the most sophisticated
      // and resembles a simplified Rust vector. Can be thought of as an array
      // of a single type.
      vector::push_back(s, 1);
      vector::push_back(s, 2);
      vector::push_back(s, 3);
    }

    // Similar to above, except removing state.
    public fun remove_stuff(sender: &signer) acquires State{
      assert!(is_testnet(), error::invalid_state(ETESTNET));
      let st = borrow_global_mut<State>(signer::address_of(sender));
      let s = &mut st.hist;

      vector::pop_back<u8>(s);
      vector::pop_back<u8>(s);
      vector::remove<u8>(s, 0);
    }

    // Here are examples of read operations. Note the `aquires` here again.
    public fun isEmpty(sender: &signer): bool acquires State {
      assert!(is_testnet(), error::invalid_state(ETESTNET));

      // Note this is not a mutable borrow. Read only.
      let st = borrow_global<State>(signer::address_of(sender));
      vector::is_empty(&st.hist)
    }

    // Showing the vector::length method
    public fun length(sender: &signer): u64 acquires State{
      assert!(is_testnet(), error::invalid_state(ETESTNET));
      let st = borrow_global<State>(signer::address_of(sender));
      vector::length(&st.hist)
    }

    // Showing the vector::contains method
    public fun contains(sender: &signer, num: u8): bool acquires State {
      assert!(is_testnet(), error::invalid_state(ETESTNET));
      let st = borrow_global<State>(signer::address_of(sender));
      vector::contains(&st.hist, &num)
    }
}