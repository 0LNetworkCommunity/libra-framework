
module ol_framework::vouch {
    use std::signer;
    use std::vector;
    use std::error;
    use ol_framework::ancestry;
    use ol_framework::ol_account;
    use ol_framework::epoch_helper;

    use diem_framework::account;
    use diem_framework::system_addresses;
    use diem_framework::transaction_fee;

    friend diem_framework::genesis;
    friend ol_framework::validator_universe;
    friend ol_framework::proof_of_fee;
    friend ol_framework::jail;
    friend ol_framework::reputation;
    friend ol_framework::epoch_boundary;

    #[test_only]
    friend ol_framework::mock;
    #[test_only]
    friend ol_framework::test_pof;

    //////// CONST ////////

    /// Maximum number of vouches
    const BASE_MAX_VOUCHES: u64 = 2;

    /// how many epochs must pass before the voucher expires.
    // Commit Note: proposing faster vouch expiration
    const EXPIRATION_ELAPSED_EPOCHS: u64 = 45;

    /// how deep the VouchTree needs to be
    const VOUCH_TREE_DEPTH: u64 = 1;

    //////// ERROR CODES ////////

    /// Limit reached. You cannot give any new vouches.
    const EMAX_LIMIT_GIVEN: u64 = 4;

    /// trying to vouch for yourself?
    const ETRY_SELF_VOUCH_REALLY: u64 = 1;

    /// User cannot receive vouch because their vouch state is not initialized
    const ERECEIVER_NOT_INIT: u64 = 2;

    /// Cannot give vouch because your vouch state is not initialized
    const EGIVER_STATE_NOT_INIT: u64 = 3;


    /// Struct for the Voucher
    /// Alice vouches for Bob, this is Alice's state
    struct GivenOut has key {
      /// list of the accounts I'm vouching for
      vouches_given: vector<address>,
      /// maximum number of vouches that can be given
      limit: u64,
    }

    /// Struct for the Vouchee
    /// Vouches received from other people
    /// keeps a list of my buddies and when the vouch was given.
    /// Alice vouches for Bob, this is Bob's state

    // TODO: someday this should be renamed to Received
    struct MyVouches has key {
      my_buddies: vector<address>,
      epoch_vouched: vector<u64>,
    }

    // a group of validators N hops away by vouch
    struct Cohort has store, drop {
      list: vector<address>
    }

    // the successive cohorts of validators at each hop away. 0th element is first
    // hop.
    struct VouchTree has key {
      // all upstream vouches
      received_from_upstream: vector<Cohort>,
      // all downstream vouches
      given_to_downstream: vector<Cohort>,
    }

    // init the struct on a validators account.
    public(friend) fun init(new_account_sig: &signer) {
      let acc = signer::address_of(new_account_sig);

      if (!exists<MyVouches>(acc)) {
        move_to<MyVouches>(new_account_sig, MyVouches {
          my_buddies: vector::empty(),
          epoch_vouched: vector::empty(),
        });
      };
      // Note: separate initialization for migrations of accounts which already
      // have MyVouches
      if(!exists<GivenOut>(acc)){
        move_to<GivenOut>(new_account_sig, GivenOut {
          vouches_given: vector::empty(),
          limit: 0,
        });
      };

      if (!exists<VouchTree>(acc)) {
        move_to(new_account_sig, VouchTree {
          received_from_upstream: vector::empty(),
          given_to_downstream: vector::empty(),
        })
      }
    }


    #[view]
    // NOTE: this should be renamed to is_received_init()
    public fun is_init(acc: address ):bool {
      exists<MyVouches>(acc)
    }

    // implement the vouching.
    fun vouch_impl(give_sig: &signer, receive_acc: address) acquires MyVouches, GivenOut, VouchTree {
      // heal the account if there is a migration issue:
      init(give_sig);

      let give_acc = signer::address_of(give_sig);
      assert!(give_acc != receive_acc, error::invalid_argument(ETRY_SELF_VOUCH_REALLY));

      assert!(exists<MyVouches>(receive_acc), error::invalid_state(ERECEIVER_NOT_INIT));

      // this fee is paid to the system, cannot be reclaimed
      // fail fast if no fee can be paid
      let c = ol_account::withdraw(give_sig, vouch_cost_microlibra());
      transaction_fee::user_pay_fee(give_sig, c);

      // add to receipient's state first
      let v = borrow_global_mut<MyVouches>(receive_acc);
      add_or_refresh_buddy_to_recipient(v, give_acc);

      // check if we reached our max number of vouches given
      // Note: we only check if we try to add a new buddy.
      // will not fail if we are just extending the expiration
      checked_add_buddy_to_giver(give_sig, receive_acc);

      // update vouch trees for both giver and receiver.
      construct_vouch_tree(give_acc, true, VOUCH_TREE_DEPTH);
      construct_vouch_tree(give_acc, false, VOUCH_TREE_DEPTH);

      construct_vouch_tree(receive_acc, true, VOUCH_TREE_DEPTH);
      construct_vouch_tree(receive_acc, false, VOUCH_TREE_DEPTH);

    }

    fun checked_add_buddy_to_giver(give_sig: &signer, receive_acc: address) acquires GivenOut {
      let give_acc = signer::address_of(give_sig);
      let give_state = borrow_global_mut<GivenOut>(give_acc);

      // check this account is not already on the list.
      // error if already there
      let (found, _i) = vector::index_of(&give_state.vouches_given, &receive_acc);
      if (!found) { // prevent duplicates
        assert!(can_add(give_state), error::invalid_state(EMAX_LIMIT_GIVEN));
        vector::push_back(&mut give_state.vouches_given, receive_acc);
      }
    }

    // receipient's state gets updated.
    // guarded function since the signer cannot write state of recipient,
    // besides in private function.
    fun add_or_refresh_buddy_to_recipient(receive_state: &mut MyVouches, give_acc: address) {
      let epoch = epoch_helper::get_current_epoch();

      let (found, i) = vector::index_of(&receive_state.my_buddies, &give_acc);
      if (found) { // prevent duplicates
        // update date
        let e = vector::borrow_mut(&mut receive_state.epoch_vouched, i);
        *e = epoch;
      } else {
        // limit amount of vouches given to 3
        vector::insert(&mut receive_state.my_buddies, 0, give_acc);
        vector::insert(&mut receive_state.epoch_vouched, 0, epoch);
      }
    }

    #[view]
    /// check if an address can add a new vouchee recipient
    public fun giver_can_add(give_acc: address): bool acquires GivenOut{
      if (!exists<GivenOut>(give_acc)) return false;

      can_add(borrow_global<GivenOut>(give_acc))
    }

    // helper to check can add vouchee
    fun can_add(give_state: &GivenOut): bool {
      vector::length(&give_state.vouches_given) < give_state.limit
    }

    /// ensures no vouch list is greater than
    /// hygiene for the vouch list
    //  TODO: this is not used anywhere, perhaps it needs to be done during the
    //  upgrade

    public(friend) fun vm_migrate_trim_vouchers(framework: &signer, give_acc: address) acquires MyVouches, GivenOut, VouchTree {
      system_addresses::assert_ol(framework);
      {
        let give_state = borrow_global_mut<GivenOut>(give_acc);
        maybe_trim_given_vouches(give_state, give_acc)
      };

      construct_vouch_tree(give_acc, true, VOUCH_TREE_DEPTH);
      construct_vouch_tree(give_acc, false, VOUCH_TREE_DEPTH);
    }

    // safely trims vouch list, drops backmost elements
    fun maybe_trim_given_vouches(give_state: &mut GivenOut, give_acc: address) acquires MyVouches {
        if (vector::length(&give_state.vouches_given) > give_state.limit) {
          let dropped = vector::trim(&mut give_state.vouches_given, give_state.limit - 1);
          vector::for_each(dropped, |addr| {
            revoke_impl(give_state, give_acc, addr);
          })
      }
    }

    /// will only succesfully vouch if the two are not related by ancestry
    /// prevents spending a vouch that would not be counted.
    /// to add a vouch and ignore this check use insist_vouch
    public entry fun vouch_for(give_sig: &signer, receive: address) acquires MyVouches, GivenOut, VouchTree {
      ancestry::assert_unrelated(signer::address_of(give_sig), receive);
      vouch_impl(give_sig, receive);
    }

    /// you may want to add people who are related to you
    /// there are no known use cases for this at the moment.
    public entry fun insist_vouch_for(give_sig: &signer, receive: address) acquires MyVouches, GivenOut, VouchTree {
      vouch_impl(give_sig, receive);
    }

    /// Let's break up with this account
    public entry fun revoke(give_sig: &signer, its_not_me_its_you: address) acquires MyVouches, GivenOut, VouchTree  {
      let give_acc = signer::address_of(give_sig);

      // Commit note: this check is not necessary, if this did somehow happen we
      // would need it to self heal.
      // assert!(give_acc!=its_not_me_its_you, ETRY_SELF_VOUCH_REALLY);

      assert!(exists<MyVouches>(its_not_me_its_you), error::invalid_state(ERECEIVER_NOT_INIT));
      assert!(exists<GivenOut>(give_acc), error::invalid_state(EGIVER_STATE_NOT_INIT));

      let given_state = borrow_global_mut<GivenOut>(give_acc);

      revoke_impl(given_state, give_acc, its_not_me_its_you);

      // update vouch trees for both giver and receiver.
      construct_vouch_tree(give_acc, true, VOUCH_TREE_DEPTH);
      construct_vouch_tree(give_acc, false, VOUCH_TREE_DEPTH);

      construct_vouch_tree(its_not_me_its_you, true, VOUCH_TREE_DEPTH);
      construct_vouch_tree(its_not_me_its_you, false, VOUCH_TREE_DEPTH);
    }

    fun revoke_impl(given_state: &mut GivenOut, give_acc: address, receive_acc: address) acquires MyVouches{

      // first update the recipient's state
      let v = borrow_global_mut<MyVouches>(receive_acc);
      let (found, i) = vector::index_of(&v.my_buddies, &give_acc);
      if (found) {
        vector::remove(&mut v.my_buddies, i);
        vector::remove(&mut v.epoch_vouched, i);
      };

      // next maybe we need to clean up the giver's state
      // Though possibly this has already been trimmed
      let (found, i) = vector::index_of(&given_state.vouches_given, &receive_acc);
      if (found) {
        vector::remove(&mut given_state.vouches_given, i);
      };

    }

    /// If we need to reset a vouch list for genesis and upgrades
    public(friend) fun vm_migrate(vm: &signer, val: address, buddy_list: vector<address>) acquires MyVouches {
      system_addresses::assert_ol(vm);
      bulk_set(val, buddy_list);
    }

    public(friend) fun vm_migrate_vouch_tree(framework: &signer, vals: vector<address>) acquires MyVouches, GivenOut, VouchTree {
      system_addresses::assert_diem_framework(framework);
      batch_vouch_tree(vals);
    }

    fun batch_vouch_tree(vals: vector<address>) acquires MyVouches, GivenOut, VouchTree {
      vector::for_each(vals, |acc| {
        construct_vouch_tree(acc, true, VOUCH_TREE_DEPTH);
        construct_vouch_tree(acc, false, VOUCH_TREE_DEPTH);
      })
    }

    fun construct_vouch_tree(validator: address, up_or_downstream: bool, iters: u64) acquires MyVouches, GivenOut, VouchTree {
      if (
        !exists<MyVouches>(validator) ||
        !exists<GivenOut>(validator) ||
        !exists<VouchTree>(validator)
      ) return;
      // NOTE: upstream's 0th element is a copy of the my_buddies struct.
      // Similarly, downstream's first element is a copy of the GivenOut.list
      // TODO: someday consider deduplicating this.
      let start_list = if (up_or_downstream) {
        borrow_global<MyVouches>(validator).my_buddies
      } else {
        borrow_global<GivenOut>(validator).vouches_given
      };

      let first_cohort = Cohort {
        list: start_list
      };

      let cohort_vec = vector::singleton(first_cohort);

      let i = 0;
      while (i < iters) {
        let this_cohort = vector::borrow(&cohort_vec, i);
        let this_hop_addrs = this_cohort.list;

        let next_hop_addrs = vector::empty<address>();

        vector::for_each(this_hop_addrs, |buddy| {
          let v = vector::empty();

          if (up_or_downstream) {
            if (exists<MyVouches>(buddy)) {
              v = borrow_global<MyVouches>(buddy).my_buddies
            }
          } else {
            if (exists<GivenOut>(buddy)) {
              v = borrow_global<GivenOut>(buddy).vouches_given
            }
          };
          vector::append(&mut next_hop_addrs, v)
        });
        let next_cohort = Cohort {
          list: next_hop_addrs
        };
        vector::push_back(&mut cohort_vec, next_cohort);
        i = i + 1;
      };

      let state = borrow_global_mut<VouchTree>(validator);
      if (up_or_downstream) {
        state.received_from_upstream = cohort_vec;
      } else {
        state.given_to_downstream = cohort_vec;
      }
    }

    // implements bulk setting of vouchers
    fun bulk_set(receiver_acc: address, buddy_list: vector<address>) acquires MyVouches {

      if (!exists<MyVouches>(receiver_acc)) return;

      let v = borrow_global_mut<MyVouches>(receiver_acc);

      // take self out of list
      let (is_found, i) = vector::index_of(&buddy_list, &receiver_acc);

      if (is_found) {
        vector::swap_remove<address>(&mut buddy_list, i);
      };

      v.my_buddies = buddy_list;

      let epoch_data: vector<u64> = vector::map_ref(&buddy_list, |_e| { 0u64 } );
      v.epoch_vouched = epoch_data;
    }


    public(friend) fun get_cohort(acc: address, up_or_down: bool, hop: u64): vector<address> acquires VouchTree {
      let state = borrow_global<VouchTree>(acc);
      let list = vector::empty();
      if (up_or_down) {
        if (vector::length(&state.received_from_upstream) >= hop) {
          let c = vector::borrow(&state.received_from_upstream, hop);
          list = c.list;
        }
      } else {
        if (vector::length(&state.given_to_downstream) >= hop) {
          let c = vector::borrow(&state.given_to_downstream, hop);
          list = c.list;

        }
      };
      return list
    }

    #[view]
    /// gets all buddies, including expired ones
    public fun all_vouchers(val: address): vector<address> acquires MyVouches{

      if (!exists<MyVouches>(val)) return vector::empty<address>();
      let state = borrow_global<MyVouches>(val);
      *&state.my_buddies
    }

    #[view]
    /// gets the buddies and checks if they are expired
    public fun all_not_expired(addr: address): vector<address> acquires MyVouches{
      let valid_vouches = vector::empty<address>();
      if (is_init(addr)) {
        let state = borrow_global<MyVouches>(addr);
        vector::for_each(state.my_buddies, |buddy_acc| {
          // account might have dropped
          if (account::exists_at(buddy_acc)){
            if (is_not_expired(buddy_acc, state)) {
              vector::push_back(&mut valid_vouches, buddy_acc)
            }
          }

        })
      };
      valid_vouches
    }

    #[view]
    /// filter expired vouches, and do ancestry check
    public fun true_friends(addr: address): vector<address> acquires MyVouches{

      if (!exists<MyVouches>(addr)) return vector::empty<address>();
      let not_expired = all_not_expired(addr);
      let filtered_ancestry = ancestry::list_unrelated(not_expired);
      filtered_ancestry
    }

    #[view]
    /// check if the user is in fact a valid voucher
    public fun is_valid_voucher_for(voucher: address, recipient: address):bool
    acquires MyVouches {
      let list = true_friends(recipient);
      vector::contains(&list, &voucher)
    }


    fun is_not_expired(voucher: address, state: &MyVouches): bool {
      let (found, i) = vector::index_of(&state.my_buddies, &voucher);
      if (found) {
        let when_vouched = vector::borrow(&state.epoch_vouched, i);
        return  (*when_vouched + EXPIRATION_ELAPSED_EPOCHS) > epoch_helper::get_current_epoch()
      };
      false
    }

    /// for a given list find and count any of my vouchers
    public(friend) fun true_friends_in_list(addr: address, list: &vector<address>): (vector<address>, u64) acquires MyVouches {

      if (!exists<MyVouches>(addr)) return (vector::empty(), 0);

      let tf = true_friends(addr);

      let buddies_in_list = vector::empty();
      let  i = 0;
      while (i < vector::length(&tf)) {
        let addr = vector::borrow(&tf, i);

        if (vector::contains(list, addr)) {
          vector::push_back(&mut buddies_in_list, *addr);
        };
        i = i + 1;
      };

      (buddies_in_list, vector::length(&buddies_in_list))
    }

    /// Root account can set the max limit to vouches that can be given by this account
    public(friend) fun set_limit(framework: &signer, give_acc: address, limit: u64) acquires GivenOut{
      system_addresses::assert_diem_framework(framework);
      let give_state = borrow_global_mut<GivenOut>(give_acc);
      give_state.limit = limit;
    }

    // TODO: move to globals
    // the cost to verify a vouch. Coins are burned.
    fun vouch_cost_microlibra(): u64 {
      1000
    }

    #[test_only]
    public fun test_set_buddies(val: address, buddy_list: vector<address>) acquires MyVouches {
      bulk_set(val, buddy_list);
    }
  }
