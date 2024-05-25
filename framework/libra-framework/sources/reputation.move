/// Validator track record
/// NOTE: other reputational metrics exist on jail.move
module diem_framework::reputation {
  use std::signer;
  use std::vector;
  use ol_framework::leaderboard;
  use ol_framework::jail;

  friend ol_framework::validator_universe;

  const BASE_REPUTATION: u64 = 20; // 2 stars out of five, one decimal

  struct Reputation has key {
    score_baseline: u64,
    // for every iteration/hop of the graph calculate the recursive score
    // 0th score is first level of recursion.
    score_upstream: vector<u64>,
    score_downstream: vector<u64>
  }

  // a group of validators N hops away from a validator
  struct Cohort has store {
    list: vector<address>
  }

  // the successive cohorts of validators at each hop away. 0th element is first
  // hop.
  struct ReputationTree has key {
    upstream_cohorts: vector<Cohort>,
    downstream_cohorts: vector<Cohort>,
  }

  public(friend) fun init(sig: &signer) {
    let addr = signer::address_of(sig);
    if (!exists<Reputation>(addr)) {
      move_to(sig, Reputation {
        score_baseline: 0,
        score_upstream: vector::empty(),
        score_downstream: vector::empty(),
      })
    };

    if (!exists<ReputationTree>(addr)) {
      move_to(sig, ReputationTree {
        upstream_cohorts: vector::empty(),
        downstream_cohorts: vector::empty(),
      })
    }
  }



  #[view]
  /// get the validator reputation index
  public fun calc_reputation(acc: address): u64 {
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
}
