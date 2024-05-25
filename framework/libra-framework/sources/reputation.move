/// Validator track record
/// NOTE: other reputational metrics exist on jail.move
module diem_framework::reputation {
  use std::signer;
  use std::vector;
  use diem_framework::system_addresses;
  use ol_framework::leaderboard;
  use ol_framework::jail;
  use ol_framework::vouch;

  friend ol_framework::validator_universe;

  const BASE_REPUTATION: u64 = 20; // 2 stars out of five, one decimal

  struct Reputation has key {
    score_baseline: u64,
    // for every iteration/hop of the graph calculate the recursive score
    // 0th score is first level of recursion.
    score_upstream: vector<u64>,
    score_downstream: vector<u64>
  }

  public(friend) fun init(sig: &signer) {
    let addr = signer::address_of(sig);
    if (!exists<Reputation>(addr)) {
      move_to(sig, Reputation {
        score_baseline: 0,
        score_upstream: vector::empty(),
        score_downstream: vector::empty(),
      })
    }
  }

  public(friend) fun batch_score(framework_sig: &signer, vals: vector<address>, iters: u64) acquires Reputation {
    system_addresses::assert_diem_framework(framework_sig);
    // must update base first
    batch_set_base(vals);
    // the downstream recursive at iters
    batch_set_recursive(vals, false, iters);
    // the upstream recursive at iters
    batch_set_recursive(vals, true, iters);
  }



  fun batch_set_recursive(vals: vector<address>, up_or_down: bool, iters: u64) acquires Reputation {
    // need to update all the addresses base scores first
    vector::for_each(vals, |acc| {
      set_recursive(acc, up_or_down, iters);
    })
  }

  fun set_recursive(acc: address, up_or_down: bool, iters: u64) acquires Reputation {

    let score_vec = vector::empty<u64>();

    let i = 0;
    while (i < iters) {
      let vals = vouch::get_cohort(acc, up_or_down, i);
      let group_total = 0;
      vector::for_each(vals, |addr| {
        let s = get_base_reputation(addr);
        group_total = group_total + s;
      });
      let len = vector::length(&vals);
      let hop_score = 0;
      if (len > 0) {
         hop_score = group_total / len;
      };
      vector::push_back(&mut score_vec, hop_score);
      i = i + 1;
    };
    let state = borrow_global_mut<Reputation>(acc);
    if (up_or_down) {
      state.score_upstream = score_vec;
    } else {
      state.score_downstream = score_vec;
    }
  }

  fun batch_set_base(vals: vector<address>) acquires Reputation {
    // need to update all the addresses base scores first
    vector::for_each(vals, |acc| {
      set_base_reputation(acc);
    })
  }

  // save the baseline reputation for a user
  fun set_base_reputation(acc: address) acquires Reputation {
      let state = borrow_global_mut<Reputation>(acc);
      let base = calc_base_reputation(acc);
      state.score_baseline = base;
  }

  /// get the validator base reputation
  fun calc_base_reputation(acc: address): u64 {
    let (wins, losses) = leaderboard::get_total(acc);
    let topten_streak = leaderboard::get_streak(acc);
    let win_streak = leaderboard::get_topten_streak(acc);
    let lifetime_vouchees_jailed = jail::get_count_buddies_jailed(acc);
    let (_, consecutive_jails) = jail::get_jail_reputation(acc);

    // unless you are winning you don't get to vouch for people.
    let rep = BASE_REPUTATION;
    // if you've been jailed more than you've won over time, this game may
    // not be for you
    if (losses > 0 && losses >  wins)  {
      return 0
    };
    // If you fall within certain thresholds of Leaderboard you get extra points.
    // If you fall below a threshold on Jail statistics you lose points

    // If you are an all star you are allowed more vouches
    if (topten_streak > 7) {
      rep = rep + 1;
    };

    // if you have been on a winning streak for one month, you are great too
    if (win_streak > 30) {
      rep = rep + 1;
    };
    let net_wins = 0;
    if (wins > 0 && wins > losses) {
      net_wins = wins - losses
    };
    // if you have a high total score, lots of games played and won  (wins -
    // losses) (even though you may be having a bad streak)
    if (net_wins > 60) {
      rep = rep + 1;
    };

    // Now for negative points
    // are you joining the validator set at an acceptable rate?
    if (rep > 0 && net_wins > 0) {
      let win_rate = (net_wins * 100) / ( wins + losses);
      if (win_rate < 80 )  {
        rep = rep - 1;
      }
    };

    // if you are repeatedly failing to stay in the validator set recently, you
    // lose reputation
    if (rep > 0 && consecutive_jails > 3) {
      rep = rep - 1;
    };

    // are the people you are vouching for getting jailed?
    if (rep > 0) {
      if (lifetime_vouchees_jailed > 30 )  {
        rep = rep - 1;
      }
    };

    return rep

  }

  //////// GETTERS ////////
  #[view]
  /// Get the baseline reputation of an address. Not a recursive reputation calc.
  public fun get_base_reputation(acc: address): u64 acquires Reputation {
    borrow_global<Reputation>(acc).score_baseline
  }

  #[view]
  /// Get the baseline reputation of an address. Not a recursive reputation calc.
  public fun get_recursive_score(acc: address, up_or_downstream: bool, hop: u64): u64 acquires Reputation {
     let state = borrow_global<Reputation>(acc);

    if (up_or_downstream) {
      return *vector::borrow(&state.score_upstream, hop)
    } else {
      return *vector::borrow(&state.score_downstream, hop)
    }
  }
}
