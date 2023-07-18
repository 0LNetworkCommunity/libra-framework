// Burn or Match
// This (experimental) mechanism allows users to set their preferences for system-wide burns.
// A system burn may exist because of rate-limiting, or collecting more fees than there are rewards to pay out. Or other third-party contracts may have a requirement to burn.
// The user can elect a simple Burn, where the coin is permanently taken out of circulation, and the total supply is adjusted downward.
// A Match (aka recycle) will keep the coin internal, submit to wallets which have opted into being a Match recipient.
// There's a longer discussion on the game theory in ol/docs/burn_match.md
// The short version is: in 0L there are no whitelists anyhwere. As such an important design goal for Burn or Match is that there is no privilege required for an account to opt it. Another design goal is that you "put your money where your mouth is" and the cost is real (you could not weight the index by "stake-based" voting, you must forfeit the coins). This does not mean that opting-in has no constraints.
/// It doesn't require any permission to be a recipient of matching funds, it's open to all (including attackers). However, this mechanism uses a market to establish which of the opt-in accounts will receive funds. And this is a simple weighting algorithm which more heavily weights toward recent donations. Evidently there are attacks possible,

// It will be costly and unpredictable (but not impossible) for attacker to close the loop, on Matching funds to themselves. It's evident that an attacker can get return on investment by re-weighting the Match Index to favor a recipient controlled by them. This is especially the case if there is low participation in the market (i.e. very few people are donating out of band).
// Mitigations: There are conditions to qualify for matching. Matching accounts must be "resource accounts" with a Donor Directed multi-sig enabled. See more under donor_directed.move, but in short there is a multisig to authorize transactions, and donors to that account can vote to delay, freeze, and ultimately liquidate the donor directed account. Since the attacker can clearly own all the multisig authorities on the Donor Directed account, there is another condition which is checked for: the multisig signers cannot be related by Ancestry, meaning the attacker needs to collect conspirators from across the network graph. Lastly, any system burns (e.g. proof-of-fee) from other honest users to that account gives them governance and possibly legal claims against that wallet, there is afterall an off-chain world.
// The expectation is that there is no additional overhead to honest actors, but considerable uncertainty and cost to abusers. Probabilistically there will be will some attacks or anti-social behavior. However, not all is lost: abusive behavior requires playing many other games honestly. The expectation of this experiment is that abuse is de-minimis compared to the total coin supply. As the saying goes: You can only prevent all theft if you also prevent all sales.

module ol_framework::burn {
  use std::fixed_point32;
  use std::signer;
  use std::vector;
  use ol_framework::donor_directed;
  use ol_framework::ol_account;
  use ol_framework::system_addresses;
  use ol_framework::gas_coin::GasCoin;
  use ol_framework::transaction_fee;
  use ol_framework::coin::{Self, Coin};
  use ol_framework::cumulative_deposits;
  use ol_framework::fee_maker;
  // use ol_framework::Debug::print;

  struct BurnPreference has key {
    send_community: bool
  }

  struct BurnState has key {
    addr: vector<address>,
    deposits: vector<u64>,
    ratio: vector<fixed_point32::FixedPoint32>,
    lifetime_burned: u64,
    lifetime_recycled: u64,
  }

  /// At the end of the epoch, after everyone has been paid
  /// subsidies (validators, oracle, maybe future infrastructure)
  /// then the remaining fees are burned or recycled
  /// Note that most of the time, the amount of fees produced by the Fee Makers
  /// is much larger than the amount of fees available burn.
  /// So we need to find the proportion of the fees that each Fee Maker has
  /// produced, and then do a weighted burn/recycle.
  public fun epoch_burn_fees(
      vm: &signer,
      total_fees_collected: u64,
  )  acquires BurnPreference, BurnState {
      system_addresses::assert_ol(vm);

      // extract fees
      let coins = transaction_fee::root_withdraw_all(vm);

      if (coin::value(&coins) == 0) {
        coin::destroy_zero(coins);
        return
      };

      // print(&coin::value(&coins));
      // get the list of fee makers
      let fee_makers = fee_maker::get_fee_makers();
      // print(&fee_makers);

      let len = vector::length(&fee_makers);

      // for every user in the list burn their fees per Burn.move preferences
      let i = 0;
      while (i < len) {
          let user = vector::borrow(&fee_makers, i);
          let amount = fee_maker::get_epoch_fees_made(*user);
          let share = fixed_point32::create_from_rational(amount, total_fees_collected);
          // print(&share);

          let to_withdraw = fixed_point32::multiply_u64(coin::value(&coins), share);
          // print(&to_withdraw);

          if (to_withdraw > 0 && to_withdraw <= coin::value(&coins)) {
            let user_share = coin::extract(&mut coins, to_withdraw);
            // print(&user_share);

            burn_with_user_preference(vm, *user, user_share);
          };


          i = i + 1;
      };

    // Transaction fee account should be empty at the end of the epoch
    // Superman 3 decimal errors. https://www.youtube.com/watch?v=N7JBXGkBoFc
    // anything that is remaining should be burned
    coin::user_burn(coins);
  }

  /// initialize, usually for testnet.
  public fun initialize(vm: &signer) {
    system_addresses::assert_vm(vm);

    move_to<BurnState>(vm, BurnState {
        addr: vector::empty(),
        deposits: vector::empty(),
        ratio: vector::empty(),
        lifetime_burned: 0,
        lifetime_recycled: 0,
      })
  }

  /// Migration script for hard forks
  public fun vm_migration(vm: &signer,
    addr_list: vector<address>,
    deposit_vec: vector<u64>,
    ratios_vec: vector<fixed_point32::FixedPoint32>,
    lifetime_burned: u64, // these get reset on final supply V6. Future upgrades need to decide what to do with this
    lifetime_recycled: u64,
  ) {

    // TODO: assert genesis when timesetamp is working again.
    system_addresses::assert_vm(vm);

    move_to<BurnState>(vm, BurnState {
        addr: addr_list,
        deposits: deposit_vec,
        ratio: ratios_vec,
        lifetime_burned,
        lifetime_recycled,
      })
  }

  /// At each epoch boundary, we recalculate the index of the Match recipients.
  public fun reset_ratios(vm: &signer) acquires BurnState {
    system_addresses::assert_ol(vm);
    let list = donor_directed::get_root_registry();

    let len = vector::length(&list);
    let i = 0;
    let global_deposits = 0;
    let deposit_vec = vector::empty<u64>();

    while (i < len) {

      let addr = *vector::borrow(&list, i);
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

    if (exists<BurnState>(@ol_framework)) {
      let d = borrow_global_mut<BurnState>(@ol_framework);
      d.addr = list;
      d.deposits = deposit_vec;
      d.ratio = ratios_vec;
    } else { // hot migration
      move_to<BurnState>(vm, BurnState {
        addr: list,
        deposits: deposit_vec,
        ratio: ratios_vec,
        lifetime_burned: 0,
        lifetime_recycled: 0,
      })
    }
  }

  fun get_address_list(): vector<address> acquires BurnState {
    if (!exists<BurnState>(@ol_framework))
      return vector::empty<address>();

    *&borrow_global<BurnState>(@ol_framework).addr
  }

  /// calculate the ratio which the matching wallet should receive per the recently weighted historical donations.
  fun get_payee_value(payee: address, value: u64): u64 acquires BurnState {
    if (!exists<BurnState>(@ol_framework))
      return 0;

    let d = borrow_global<BurnState>(@ol_framework);
    let _contains = vector::contains(&d.addr, &payee);
    let (is_found, i) = vector::index_of(&d.addr, &payee);
    if (is_found) {
      let len = vector::length(&d.ratio);
      if (i + 1 > len) return 0;
      let ratio = *vector::borrow(&d.ratio, i);
      if (fixed_point32::is_zero(copy ratio)) return 0;
      return fixed_point32::multiply_u64(value, ratio)
    };

    0
  }

  fun burn_with_user_preference(
    vm: &signer, payer: address, user_share: Coin<GasCoin>
  ) acquires BurnState, BurnPreference {
    system_addresses::assert_vm(vm);
    // print(&5050);
    if (exists<BurnPreference>(payer)) {

      if (borrow_global<BurnPreference>(payer).send_community) {
        // print(&5051);
        recycle(vm, payer, &mut user_share);

      }
    };

    // Superman 3
    let state = borrow_global_mut<BurnState>(@ol_framework);
    // print(&state.lifetime_burned);
    state.lifetime_burned = state.lifetime_burned + coin::value(&user_share);
    // print(&state.lifetime_burned);
    coin::user_burn(user_share);
  }


  fun recycle(vm: &signer, payer: address, coin: &mut Coin<GasCoin>) acquires BurnState {
    let list = { get_address_list() }; // NOTE devs, the added scope drops the borrow which is used below.
    let len = vector::length<address>(&list);
    let total_coin_value_to_recycle = coin::value(coin);

    // There could be errors in the array, and underpayment happen.
    let value_sent = 0;

    let i = 0;
    while (i < len) {

      let payee = *vector::borrow<address>(&list, i);
      // print(&payee);
      let amount_to_payee = get_payee_value(payee, total_coin_value_to_recycle);
      // let to_deposit = coin::withdraw(coin, amount_to_payee);

      ol_account::vm_transfer(
          vm,
          payer,
          payee,
          amount_to_payee,
      );
      value_sent = value_sent + amount_to_payee;
      i = i + 1;
    };

    // if there is anything remaining it's a superman 3 issue
    // so we send it back to the transaction fee account
    // makes it easier to track since we know no burns should be happening.
    // which is what would happen if the coin didn't get emptied here
    let remainder_amount = coin::value(coin);
    if (remainder_amount > 0) {
      let last_coin = coin::extract(coin, remainder_amount);
      // use pay_fee which doesn't track the sender, so we're not double counting the receipts, even though it's a small amount.
      transaction_fee::pay_fee(vm, last_coin);
    };

    // update the root state tracker
    let state = borrow_global_mut<BurnState>(@ol_framework);
    // print(&state.lifetime_recycled);
    state.lifetime_recycled = state.lifetime_recycled + value_sent;
    // print(&state.lifetime_recycled);
  }

  public fun set_send_community(sender: &signer, community: bool) acquires BurnPreference {
    let addr = signer::address_of(sender);
    if (exists<BurnPreference>(addr)) {
      let b = borrow_global_mut<BurnPreference>(addr);
      b.send_community = community;
    } else {
      move_to<BurnPreference>(sender, BurnPreference {
        send_community: community
      });
    }
  }

  //////// GETTERS ////////
  public fun get_ratios():
    (vector<address>, vector<u64>, vector<fixed_point32::FixedPoint32>) acquires BurnState
  {
    let d = borrow_global<BurnState>(@ol_framework);
    (*&d.addr, *&d.deposits, *&d.ratio)
  }

  public fun get_lifetime_tracker(): (u64, u64) acquires BurnState {
    let state = borrow_global<BurnState>(@ol_framework);
    (state.lifetime_burned, state.lifetime_recycled)
  }

  //////// TEST HELPERS ////////

}