
module ol_framework::match_index {
  use diem_framework::system_addresses;
  use ol_framework::cumulative_deposits;
  use ol_framework::libra_coin::LibraCoin;
  use ol_framework::ol_account;
  use diem_framework::coin::{Self, Coin};
  use diem_framework::transaction_fee;
  use std::fixed_point32;
  use std::vector;
  use std::signer;
  use std::error;

  /// matching index is not initialized
  const ENOT_INIT: u64 = 1;

  friend ol_framework::community_wallet;
  /// The Match index keeps accounts that have opted-in.
  struct MatchIndex has key {
    addr: vector<address>,
    index: vector<u64>, // the index of cumulative deposits: weighted in favor
    // of most recent deposits, per cumulative_deposits.move
    ratio: vector<fixed_point32::FixedPoint32>,
  }
  /// initialize, usually for testnet.
  public fun initialize(vm: &signer) {
    system_addresses::assert_ol(vm);

    move_to<MatchIndex>(vm, MatchIndex {
        addr: vector::empty(),
        index: vector::empty(),
        ratio: vector::empty(),
      })
  }

  // account can opt into match_index
  // checking for qualification should happen first in friend module
  public(friend) fun opt_into_match_index(sig: &signer) acquires MatchIndex {
    assert!(exists<MatchIndex>(@ol_framework), error::invalid_state(ENOT_INIT));
    let burn_state = borrow_global_mut<MatchIndex>(@ol_framework);
    let addr = signer::address_of(sig);
    if (!vector::contains(&burn_state.addr, &addr)) {
      vector::push_back(&mut burn_state.addr, addr);
    };
  }




  /// At each epoch boundary, we recalculate the index of the Match recipients.
  // NOTE: can't call community_wallet here to check for qualification because of cyclic dependencies, need to do it in ol_reconfig
  public(friend) fun calc_ratios(vm: &signer, qualifying: vector<address>) acquires MatchIndex {
    system_addresses::assert_ol(vm);
    // don't abort
    if (!exists<MatchIndex>(@ol_framework)) return;

    // TODO! if it doesn't qualify don't let it bias the ratios

    let burn_state = borrow_global_mut<MatchIndex>(@ol_framework);

    let len = vector::length(&qualifying);
    let i = 0;
    let global_deposits = 0;
    let deposit_vec = vector::empty<u64>();

    while (i < len) {

      let addr = *vector::borrow(&burn_state.addr, i);
      let deposit_index = cumulative_deposits::get_index_cumu_deposits(addr);

      global_deposits = global_deposits + deposit_index;
      vector::push_back(&mut deposit_vec, deposit_index);
      i = i + 1;
    };

    if (global_deposits == 0) return;

    let ratios_vec = vector::empty<fixed_point32::FixedPoint32>();
    let k = 0;
    while (k < len) {
      let cumu = *vector::borrow(&deposit_vec, k);

      let ratio = fixed_point32::create_from_rational(cumu, global_deposits);

      vector::push_back(&mut ratios_vec, ratio);
      k = k + 1;
    };

    burn_state.index = deposit_vec;
    burn_state.ratio = ratios_vec;
  }

    /// the root account can take a user coin, and match with accounts in index.
  // Note, this leave NO REMAINDER, and burns any rounding.
  // TODO: When the coin is sent, an attribution is also made to the payer.
  public fun match_and_recycle(vm: &signer, coin: &mut Coin<LibraCoin>) acquires MatchIndex {

    match_impl(vm, coin);
    // if there is anything remaining it's a superman 3 issue
    // so we send it back to the transaction fee account
    // makes it easier to track since we know no burns should be happening.
    // which is what would happen if the coin didn't get emptied here
    let remainder_amount = coin::value(coin);
    if (remainder_amount > 0) {
      let last_coin = coin::extract(coin, remainder_amount);
      // use pay_fee which doesn't track the sender, so we're not double counting the receipts, even though it's a small amount.
      transaction_fee::vm_pay_fee(vm, @ol_framework, last_coin);
    };
  }

    /// the root account can take a user coin, and match with accounts in index.
  // TODO: When the coin is sent, an attribution is also made to the payer.
  fun match_impl(vm: &signer, coin: &mut Coin<LibraCoin>) acquires MatchIndex {
    system_addresses::assert_ol(vm);
    let list = { get_address_list() }; // NOTE devs, the added scope drops the borrow which is used below.
    let len = vector::length<address>(&list);
    let total_coin_value_to_recycle = coin::value(coin);

    // There could be errors in the array, and underpayment happen.
    let value_sent = 0;

    let i = 0;
    while (i < len) {

      let payee = *vector::borrow<address>(&list, i);
      let amount_to_payee = get_payee_share(payee, total_coin_value_to_recycle);
      let coin_split = coin::extract(coin, amount_to_payee);
      ol_account::deposit_coins(
          payee,
          coin_split,
      );
      value_sent = value_sent + amount_to_payee;
      i = i + 1;
    };
  }

  //////// GETTERS ////////
  /// get the proportional share of each account qualified for Match, based on the recent-weighted donations. Returns a tuple of the table (list addresses, list of index of cumulative deposits, list fraction of total index)
  public fun get_ratios(): (vector<address>, vector<u64>, vector<fixed_point32::FixedPoint32>)
  acquires MatchIndex {
    let d = borrow_global<MatchIndex>(@ol_framework);
    (*&d.addr, *&d.index, *&d.ratio)
  }

  #[view]
  /// address list of accounts opted into match_index
  public fun get_address_list(): vector<address> acquires MatchIndex {
    if (!exists<MatchIndex>(@ol_framework))
      return vector::empty<address>();

    *&borrow_global<MatchIndex>(@ol_framework).addr
  }

  /// calculate the ratio which the matching wallet should receive per the recently weighted historical donations.
  public fun get_payee_share(payee: address, total_vale_to_split: u64): u64 acquires MatchIndex {
    if (!exists<MatchIndex>(@ol_framework))
      return 0;

    let d = borrow_global<MatchIndex>(@ol_framework);
    let _contains = vector::contains(&d.addr, &payee);
    let (is_found, i) = vector::index_of(&d.addr, &payee);
    if (is_found) {
      let len = vector::length(&d.ratio);
      if (i + 1 > len) return 0;
      let ratio = *vector::borrow(&d.ratio, i);
      if (fixed_point32::is_zero(copy ratio)) return 0;
      return fixed_point32::multiply_u64(total_vale_to_split, ratio)
    };

    0
  }

  //////// TEST HELPERS ////////
  #[test_only]
  public fun test_reset_ratio(root: &signer, addr_list: vector<address>) acquires MatchIndex{
    system_addresses::assert_ol(root);
    calc_ratios(root, addr_list);
  }

}
