module ol_framework::vouch_score {
  use std::option;
  use std::vector;
  use ol_framework::ancestry;
  use ol_framework::vouch;

  /// the threshold score for a user to be considered vouched
  const THRESHOLD_SCORE: u64 = 2;

  /// get voucher's score
  /// score is percent, out of 100
  fun calculate_voucher_score(voucher: address, user: address): u64 {
      let opt = ancestry::get_degree(voucher, user);
      if (option::is_none(&opt)) {
        return 0
      };
      let degree = *option::borrow(&opt);
      if (degree == 0 ){
        return 100
      };
      if (degree > 100) {
        return 0
      };

      100 / degree
  }


  fun get_total_vouch_score(user: address): u64 {
    // we only want the vouchers which are not expired
    // and do not belong to the same family
    let valid_vouchers = vouch::true_friends(user);
    let total_score = 0;
    let i = 0;
    while (i < vector::length(&valid_vouchers)) {
      let one_voucher = vector::borrow(&valid_vouchers, i);
      let score = calculate_voucher_score(*one_voucher, user);
      total_score = total_score + score;
      i = i + 1;
    };

    total_score
  }


  #[view]

  /// counting all vouchers and their score
  /// does it add up to 2 root of trust accounts?
  /// @return true if the user is vouched
  public fun is_voucher_score_valid(user: address): bool {
    get_total_vouch_score(user) > THRESHOLD_SCORE
  }
}
