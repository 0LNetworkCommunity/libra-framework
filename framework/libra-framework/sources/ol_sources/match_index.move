
module ol_framework::match_index {
  use aptos_framework::system_addresses;
  use ol_framework::cumulative_deposits;
  use std::fixed_point32;
  use std::vector;

  // friend ol_framework::burn;
  /// The Match index keeps accounts that have opted-in. Also keeps the lifetime_burned for convenience.
  struct MatchIndex has key {
    addr: vector<address>,
    deposits: vector<u64>,
    ratio: vector<fixed_point32::FixedPoint32>,

  }
  /// initialize, usually for testnet.
  public fun initialize(vm: &signer) {
    system_addresses::assert_ol(vm);

    move_to<MatchIndex>(vm, MatchIndex {
        addr: vector::empty(),
        deposits: vector::empty(),
        ratio: vector::empty(),
      })
  }


  /// At each epoch boundary, we recalculate the index of the Match recipients.
  public fun reset_ratios(vm: &signer) acquires MatchIndex {
    system_addresses::assert_ol(vm);
    // don't abort
    if (!exists<MatchIndex>(@ol_framework)) return;

    // it it doesn't qualify don't let it bias the ratios

    let burn_state = borrow_global_mut<MatchIndex>(@ol_framework);

    let len = vector::length(&burn_state.addr);
    let i = 0;
    let global_deposits = 0;
    let deposit_vec = vector::empty<u64>();

    while (i < len) {

      let addr = *vector::borrow(&burn_state.addr, i);
      let cumu = cumulative_deposits::get_index_cumu_deposits(addr);

      global_deposits = global_deposits + cumu;
      vector::push_back(&mut deposit_vec, cumu);
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

    burn_state.deposits = deposit_vec;
    burn_state.ratio = ratios_vec;
  }

  //////// GETTERS ////////
  public fun get_ratios():
    (vector<address>, vector<u64>, vector<fixed_point32::FixedPoint32>) acquires MatchIndex
  {
    let d = borrow_global<MatchIndex>(@ol_framework);
    (*&d.addr, *&d.deposits, *&d.ratio)
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

}