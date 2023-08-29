

  // // TODO: Fix publishing on test harness.
  // // see test _meta_import_vote.move
  // // There's an issue with the test harness, where it cannot publish the module
  // // task 2 'run'. lines 31-51:
  // // Error: error[E03002]: unbound module
  // // /var/folders/0s/7kz0td0j5pqffbc143hq52bm0000gn/T/.tmp3EAMzm:3:9
  // //
  // //      use 0x1::GUID;
  // //          ^^^^^^^^^ Invalid 'use'. Unbound module: '0x1::GUID'

  #[test_only]
  module ol_framework::turnout_tally_demo {

    use ol_framework::turnout_tally::{Self, TurnoutTally};
    use ol_framework::ballot::{Self, BallotTracker};
    use ol_framework::testnet;
    use diem_framework::account;
    use std::guid;
    use std::signer;
    use std::vector;
    use std::option::Option;


    struct Vote<D> has key {
      tracker: BallotTracker<D>,
      enrollment: vector<address>
    }

    struct EmptyType has store, drop {}

    // initialize this data on the address of the election contract
    public fun init(
      sig: &signer,

    ) {
      assert!(testnet::is_testnet(), 0);

      let tracker = ballot::new_tracker<TurnoutTally<EmptyType>>();

      move_to<Vote<TurnoutTally<EmptyType>>>(sig, Vote {
        tracker,
        enrollment: vector::empty()
      });
    }

    public fun propose_ballot_by_owner(sig: &signer, voters: u64, duration: u64):guid::ID acquires Vote {
      assert!(testnet::is_testnet(), 0);
      // let cap = guid::gen_create_capability(sig);
      let ballot_guid = account::create_guid(sig);
      let noop = EmptyType {};

      let t = turnout_tally::new_tally_struct<EmptyType>(noop, voters, duration, 0);

      let vote = borrow_global_mut<Vote<TurnoutTally<EmptyType>>>(signer::address_of(sig));

      let id = guid::id(&ballot_guid);
      ballot::propose_ballot<TurnoutTally<EmptyType>>(&mut vote.tracker, ballot_guid, t);

      id
    }

     public fun vote(sig: &signer, election_addr: address, uid: &guid::ID, weight: u64, approve_reject: bool): Option<bool> acquires Vote {
      assert!(testnet::is_testnet(), 0);
      let vote = borrow_global_mut<Vote<TurnoutTally<EmptyType>>>(election_addr);
      let ballot = ballot::get_ballot_by_id_mut<TurnoutTally<EmptyType>>(&mut vote.tracker, uid);
      let tally = ballot::get_type_struct_mut<TurnoutTally<EmptyType>>(ballot);
      turnout_tally::vote<EmptyType>(sig, tally, uid, approve_reject, weight)
    }

    public fun retract(sig: &signer, uid: &guid::ID, election_addr: address) acquires Vote {
      assert!(testnet::is_testnet(), 0);
      let vote = borrow_global_mut<Vote<TurnoutTally<EmptyType>>>(election_addr);
      let ballot = ballot::get_ballot_by_id_mut<TurnoutTally<EmptyType>>(&mut vote.tracker, uid);
      let tally = ballot::get_type_struct_mut<TurnoutTally<EmptyType>>(ballot);
      turnout_tally::retract<EmptyType>(tally, uid, sig);
    }

  }
// }