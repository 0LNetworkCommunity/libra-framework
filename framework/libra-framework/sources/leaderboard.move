/// Validator track record
/// NOTE: other reputational metrics exist on jail.move
module diem_framework::leaderboard {

  // NOTE: reputation will be a work in progress. As a dev strategy, let's break
  // up the structs into small atomic pieces so we can increment without
  // needing to migrate state when we imlpement new metrics.

  /// Count total epochs participating as validator
  /// count both successes and fails
  struct TotalGames {
    success: u64,
    fail: u64
  }

  // How many epochs has this validator been in the set successfully without
  // interruption
  // resets on the first jail
  struct ConsecutiveWins {
    value: u64
  }

  // The count of epochs the validator has been in the Top Ten by net proposal count
  struct TopTenStreak {
    value: u64

  }

}
