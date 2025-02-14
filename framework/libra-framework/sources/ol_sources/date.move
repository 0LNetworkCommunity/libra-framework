// NOTE: this is the minimal date tool for purposes of slow wallet v2
// TODO: include a proper date parsing module

module ol_framework::date {
  use std::error;
  use diem_framework::timestamp;

  friend ol_framework::lockbox;

  /// the timestamp provided is not later than the previous argument
  const ENOT_LATER_TIMESTAMP: u64 = 0;

  const SECONDS_IN_DAY: u64 = 86400;

  // @returns tuple representing the day in seconds, when called:
  // 0: the equivalent timestamp for UST midnight (start of day in unix in seconds), and
  // 1: the seconds since start of day
  public(friend) fun todays_start_seconds(): (u64, u64) {
    let timestamp = timestamp::now_seconds();
    start_of_day_seconds(timestamp)
  }

  // @return - tuple representing a day in seconds:
  // 0: the equivalent timestamp for UST midnight (start of day in unix in seconds), and
  // 1: the seconds since start of day
  public(friend) fun start_of_day_seconds(timestamp: u64): (u64, u64)  {
    let secs_since_day_start = timestamp % SECONDS_IN_DAY;
    let start_of_day = timestamp - secs_since_day_start;
    (start_of_day, secs_since_day_start)
  }

  // @return - integer for full elapsed days between timestamps
  public(friend) fun days_elapsed(early_time: u64, later_time: u64): u64  {
    assert!(later_time >= early_time, error::invalid_argument(ENOT_LATER_TIMESTAMP));

    let (day_a, _) = start_of_day_seconds(early_time);

    let (day_b, _) = start_of_day_seconds(later_time);

    if (day_b > day_a) {
      let elapsed_secs = day_b - day_a;
      return elapsed_secs / SECONDS_IN_DAY
    };
    return 0
  }



  #[test(framework = @0x1)]
  fun test_day(framework: &signer) {

    timestamp::set_time_has_started_for_testing(framework);
    let ill_be_right_with_you_usecs = 1727122878;

    // system time uses unix microsecs
    timestamp::update_global_time_for_test(ill_be_right_with_you_usecs * 1000000);
    let (todays_midnight, secs_since_todays_midnight) = todays_start_seconds();
    // after two shakes of a lambs tail
    let some_time_later_in_the_same_day_usecs = 1727123437;

    // these are in fact different times.
    assert!(ill_be_right_with_you_usecs != some_time_later_in_the_same_day_usecs, 7357000);

    // again, system time uses unix microsecs
    timestamp::update_global_time_for_test(some_time_later_in_the_same_day_usecs * 1000000);

    let (maybe_a_different_day_midnight, maybe_different_secs_after_midnight) = todays_start_seconds();
    // The day is the same, so it started on the same timestamp
    assert!(todays_midnight == maybe_a_different_day_midnight, 7357001);
    // but the seconds since the start of the day should not be the same.
    assert!(secs_since_todays_midnight != maybe_different_secs_after_midnight, 7357002);

    // indeed the days elapsed should be zero
    let days = days_elapsed(ill_be_right_with_you_usecs, some_time_later_in_the_same_day_usecs);

    assert!(days == 0, 7357003);

    // soon, real soon
    let later = 1739553701;
    let more_days = days_elapsed(some_time_later_in_the_same_day_usecs, later);

    assert!(more_days == 144, 7357004);

    // as used in lockbox
    let (start_today, _) = todays_start_seconds();
    let these_days = days_elapsed(start_today, later);
    assert!(these_days == 144, 7357005);

  }
}
