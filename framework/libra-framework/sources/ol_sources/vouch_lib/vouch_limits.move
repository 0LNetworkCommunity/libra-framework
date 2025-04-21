
module ol_framework::vouch_limits {
    use std::error;
    use std::vector;
    use ol_framework::page_rank_lazy;
    use ol_framework::root_of_trust;
    use ol_framework::epoch_helper;
    use ol_framework::vouch;

    friend ol_framework::vouch_txs;

    /// Maximum number of vouches
    const BASE_MAX_VOUCHES: u64 = 20;

    /// Maximum number of vouches allowed to be given per epoch
    const MAX_VOUCHES_PER_EPOCH: u64 = 1;

    // Add these constants for revocation limits
    /// Maximum number of revocations allowed in a single epoch
    const MAX_REVOCATIONS_PER_EPOCH: u64 = 2;

    /// Cooldown period (in epochs) required after a revocation before giving a new vouch
    const REVOCATION_COOLDOWN_EPOCHS: u64 = 3;

    //////// ERRORS ////////
    /// Revocation limit reached. You cannot revoke any more vouches in this epoch.
    const EREVOCATION_LIMIT_REACHED: u64 = 7;

    /// Cooldown period after revocation not yet passed
    const ECOOLDOWN_PERIOD_ACTIVE: u64 = 8;

    /// Vouch limit reached: above max ceiling
    const EMAX_LIMIT_GIVEN_CEILING: u64 = 9;

    /// Vouch limit reached: above number of vouches received
    const EMAX_LIMIT_GIVEN_BY_RECEIPT: u64 = 10;

    /// Vouch limit reached: because of quality of your voucher
    const EMAX_LIMIT_GIVEN_BY_SCORE: u64 = 11;

    /// Vouch limit reached: too many given in current epoch
    const EMAX_VOUCHES_PER_EPOCH: u64 = 12;


    /// GIVEN VOUCHES CHECK
    /// The maximum number of vouches which can be given
    /// is the lowest of three numbers:
    /// a. below the max safety threshold of the system
    /// b. below the count of active vouches received + 1
    /// c. below the max vouches as calculated by the users' vouch score quality
    /// d. no more than 1 vouch per epoch

    // check if we are trying to update the vouching of an existing account
    // or adding a new vouch to a new account
    public(friend) fun assert_under_limit(grantor_acc: address, vouched_account: address) {
      let given_vouches = vouch::get_given_vouches_not_expired(grantor_acc);

      let (found, _i) = vector::index_of(&given_vouches, &vouched_account);

      let is_root = root_of_trust::is_root_at_registry(@diem_framework, grantor_acc);
      // don't check max vouches if we are just extending the expiration
      if (!found && !is_root) {
        // are we hitting the limit of max vouches
        assert_all_checks(grantor_acc);
      } else if (is_root) {
        // exempt from scoring, and epoch limits
        // but others apply
        // assert_received_limit_vouches(grantor_acc);
        assert_revoke_cooldown_period(grantor_acc);
        assert_safety_ceiling_vouches(grantor_acc);

      }
    }

    fun assert_all_checks(grantor_acc: address) {
      assert_max_vouches_by_score(grantor_acc);
      // assert_received_limit_vouches(grantor_acc);
      assert_epoch_vouches_limit(grantor_acc);
      // prevents user from revoking many and subsequently adding
      // to bypass the limits
      assert_revoke_cooldown_period(grantor_acc);
      assert_safety_ceiling_vouches(grantor_acc);

    }

    fun assert_safety_ceiling_vouches(grantor_acc: address) {
        // Get the received vouches that aren't expired
        let given_vouches = vouch::get_given_vouches_not_expired(grantor_acc);

        assert!(vector::length(&given_vouches) <= BASE_MAX_VOUCHES, error::invalid_state(EMAX_LIMIT_GIVEN_CEILING));
    }

      fun assert_received_limit_vouches(account: address) {
        // Get the received vouches that aren't expired
        let received_vouches = vouch::true_friends(account);
        let received_count = vector::length(&received_vouches);
        let given_vouches = vouch::get_given_vouches_not_expired(account);

        let is_root = root_of_trust::is_root_at_registry(@diem_framework, account);

        // Base case: Always allow at least vouches received + 1
        // Though root of trust accounts need to propagate trust faster
        let max_allowed = if (is_root) {
          received_count * 2
        } else {
          received_count + 1
        };

        assert!(vector::length(&given_vouches) <= max_allowed, error::invalid_state(EMAX_LIMIT_GIVEN_BY_RECEIPT));
    }

    // a user should not be able to give more vouches than their quality score allows
    fun assert_max_vouches_by_score(grantor_acc: address) {
      // check if the grantor has already reached the limit of vouches
      let given_vouches = vouch::get_given_vouches_not_expired(grantor_acc);
      let max_allowed = calculate_score_limit(grantor_acc);

      assert!(
        vector::length(&given_vouches) <= max_allowed,
        error::invalid_state(EMAX_LIMIT_GIVEN_BY_SCORE)
      );
    }



    // Check if user has already given the maximum number of vouches allowed per epoch
    fun assert_epoch_vouches_limit(grantor_acc: address) {
      let given_this_epoch = vouch::get_given_this_epoch(grantor_acc);

      // Check if user has exceeded vouches in this epoch
      assert!(
        given_this_epoch < MAX_VOUCHES_PER_EPOCH,
        error::invalid_state(EMAX_VOUCHES_PER_EPOCH)
      );
    }

    /// Calculate the maximum number of vouches a user should be able to give based on their trust score
    public fun calculate_score_limit(grantor_acc: address): u64 {
        // Calculate the quality using the social distance method
        // This avoids dependency on page_rank_lazy
        let trust_score = page_rank_lazy::get_trust_score(grantor_acc);

        // For accounts with low quality vouchers,
        // we restrict further how many they can vouch for
        let max_allowed = 1;

        // TODO: collect analytics data to review this
        if (trust_score >= 2 && trust_score < 10) {
            max_allowed = 3;
        } else if (trust_score >= 10 && trust_score < 100) {
            max_allowed = 5;
        } else if (trust_score >= 100 && trust_score < 250) {
            max_allowed = 10;
        } else if (trust_score >= 250 && trust_score < 500) {
            max_allowed = 15;
        };
        max_allowed
    }

    /// REVOCATION CHECKS
    /// within a period a user might try to add and revoke
    /// many users. As such there are some checks to make on
    /// revocation.
    /// 1. Over a lifetime of the account you cannot revoke more
    /// than you have vouched for.
    /// 2. You cannot revoke more times than the current
    /// amount of vouches you currently have received.

    public(friend) fun assert_revoke_limit(grantor_acc: address) {
      let revokes = vouch::get_revocations_this_epoch(grantor_acc);

      // Check if user has exceeded revocations in this epoch
      assert!(
        revokes < MAX_REVOCATIONS_PER_EPOCH,
        error::invalid_state(EREVOCATION_LIMIT_REACHED)
      );
    }

    // Check if enough time has passed since last revocation before giving a new vouch
    fun assert_revoke_cooldown_period(grantor_acc: address) {
      assert!(
        cooldown_period_passed(grantor_acc),
        error::invalid_state(ECOOLDOWN_PERIOD_ACTIVE)
      );
    }


    /// Helper function to check if the cooldown period has passed
    fun cooldown_period_passed(grantor_acc: address): bool {
      let current_epoch = epoch_helper::get_current_epoch();
      // FOR TESTNET:
      if (current_epoch < REVOCATION_COOLDOWN_EPOCHS) {
        return true
      };

      let last_revocation_epoch = vouch::get_last_revocation_epoch(grantor_acc);

      // Check if enough epochs have passed since last revocation
      current_epoch >= last_revocation_epoch + REVOCATION_COOLDOWN_EPOCHS
    }


    #[view]
    /// Returns the number of vouches a user can still give based on system limits.
    /// This takes into account all constraints:
    /// 1. Base maximum limit (10 vouches)
    /// 2. Score-based limit
    /// 3. Received vouches + 1 limit
    /// 4. Per-epoch limit
    /// The returned value is the minimum of all these limits minus current given vouches.
    public fun get_vouch_limit(addr: address): u64 {

      // Check if account is initialized
      if (!vouch::is_init(addr)) {
        return 0
      };

      let given_count = vector::length(&vouch::get_given_vouches_not_expired(addr));

      // check what the score would allow.
      let vouches_allowed = calculate_score_limit(addr);
      // root users exempt from the score limit
      if (root_of_trust::is_root_at_registry(@diem_framework, addr)) {
        vouches_allowed = BASE_MAX_VOUCHES;
      };

      // // check based on how many received
      // // Received limit: non-expired received vouches + 1
      // let true_friends = vouch::true_friends(addr);
      // let received_limit = vector::length(&true_friends) + 1;


      // // find the lowest number, most restrictive limit
      // let vouches_allowed = if (score_limit < received_limit) {
      //   score_limit
      // } else {
      //   received_limit
      // };

      vouches_allowed = if (vouches_allowed < BASE_MAX_VOUCHES) {
        vouches_allowed
      } else {
        BASE_MAX_VOUCHES
      };


      // Calculate remaining vouches
      if (given_count >= vouches_allowed) {
        0
      } else {
        vouches_allowed - given_count
      }
    }

}
