///////////////////////////////////////////////////////////////////
// 0L Module
// sacred_cows
///////////////////////////////////////////////////////////////////

/// Parameters which are fundamental to the principles of the community.
/// Ordinarily these parameters could be a simple one-line change in code,
/// or one transaction to the database. TL;DR they should be a major pain
/// in the ass to change.

/// All blockchains have foundational ideas. Some of them are economic
/// narratives. These narratives inform policies which are taken to be
/// existential. But all policies are written by humans and can be changed.
/// In Bitcoin, there is the sacred cow of "halving" the mining difficulty
// at a certain rate. This constant, if altered, would completely change the
/// expectations of all involved. But most importantly, if the expectations
/// aren't aligned during a change, the ensuing revolt will melt the trust in
/// the institution.

/// Trust in blockchains, is not guaranteed by math. Math provides some
/// guarantees. The root of trust is the process in which your sacred cows
/// are herded.

/// As such your unshakeable existential constant numbers need to be
/// - prominently displayed, instead of being opaque or otherwise hidden.
/// - force a multi-party negotiation on fundamental changes.
/// - the lowest friction change cannot be in the hands of a single class
/// in the balance of powers (core dev, validators, etc.).
/// - or otherwise guarded from:
/// -- a minority revolt,
/// -- malicious attack (unlikely)
/// -- developer error (hi),
/// -- technocratic authority (hi again),
/// -- well intentioned but naive interventions (said the previous guy).

/// This Module has a few non-engineering related goals, it should be:
/// - readable by a non-technical audience
/// - didactic: engineers should understand the reasoning behind the constant
/// - hard to change without being noticed
/// - reinforce social consensus: it's a document that is an extension and
/// affirmation of canonical documents and decisions (and doesn't override).

/// What it is not:
/// - code-as-law: it is not the source of truth. It explains norms in
/// different words.
/// - ergonomic: it's not supposed to make the engineering team's work
/// easier. To the contrary.
/// - performant: structuring code this way makes the functions that call it much slower. That's ok, since they are rarely called.

/// How does this work technically:
/// All SacredCows have a few tight couplings which all have to be changed in a
/// coordinated way, otherwise the code will fail, or the blockchain might fail.
/// The aim is to make sure that any changes here must be done by a large group,
/// and as such this structure will force a multi-party negotiation.

/// Note: it's always possible for a rogue validator set to halt the blockchain,
/// flash the state, create a discontinuity between blocks) and resume the
/// network. We just assume this is very noticeable.

/// Every time a function needs the number, the code checks if the
/// `constant` matches the value in the `struct`.
/// A constant which is instantiated as a SacredCow must be changed in a few ways:
/// 1. There is a hard coded Move `constant` which has the number. This ordinarily would be the baseline.
/// 2. There is a data structure a Move `struct`  which is stored in the global database.
/// 3. The `struct` does not have any updating functions, and it is stored on
/// the 0x2 address, which does not have any `signer` authority instantiated
/// elsewhere in the Move framework. That is: it cannot be updated by a simple
/// diem_governance call, which only has @framework or 0x1 privilege. A
/// function to create a 0x2 signer have to be written and deployed in
/// an upgrade. This is  trivial, but obvious to detect. NOTE: devs, helpers //
/// using create_signer for
/// 0x2, should never be made available in Move outside of genesis.move
/// #[test_only].
/// 4. Each `struct` is instantiated by a phantom struct type, which makes it unique,
/// 5. A separate formal verification `spec` exists also tightly coupled to
/// names an values. Mostly as another touch point to make it a pain.

/// Evidently, there are workarounds, and mainly this process exists to make it
/// a pain in the ass for well-meaning people to make changes.
/// But for more sneaky behavior, there's an easy workaround if the
/// points above were the ony guardrails.
/// Someone wanting to quietly change a parameter could just disable the
/// check in the Move code alone, and update the `constant` field.
/// So this value must also be checked in Rust environment.
/// 1. they need a public View function which returns the `constant`
/// 2. Fullnode, Clients, Validator, Explorer, software can directly read
/// the `Struct` in database, and compare against the `constant`.
/// E.g. a Validator node implementation can fail when it notices a change.

module ol_framework::sacred_cows {
  use std::error;
  use std::signer;
  use diem_framework::chain_status;

  friend diem_framework::genesis;

  //////// ERROR CODES ////////
  /// Something sneaky is going on.
  const EHANDS_OFF_MY_COW: u64 = 0;
  /// you need to be 0x2 to sign for this
  const ENOT_OX2: u64 = 1;
  /// already initialized
  const EALREADY_INITIALIZED: u64 = 2;

  //////// SACRED COW HARD CODING ////////
  /// how much each slow wallet gets unlocked at every epoch. Read below.
  ///////////////////////////////////////////
  /// READ ABOVE RE V7 MIGRATION AND SETTING THIS PARAM
  const SLOW_WALLET_EPOCH_DRIP: u64 = 35000 * 1000000; // COINS * SCALING FACTOR
  ///////////////////////////////////////////
  /// [SLOW_WALLET_EPOCH_DRIP] Slow Wallets are one of the principal
  ///economic innovations from 0L.
  /// How to reward the highest level of work with more benefits,
  /// while keeping the games fun for all in the immature days of the network:
  /// i.e. not allowing the whales to sweep every game.
  /// This is a political choice. We don't make claims about how this works in
  /// society. We have just come across a balance which seems both attractive to
  /// high performing entrepreneurs, their enablers (partners, families,
  /// customers, etc who will have lower participation), and the public at
  /// large which are casually connected to the community.

  /// There are also very practical reasons for why an early network doesn't
  /// have all the coins unrestricted for all uses at all times. Errors,
  /// exploits, and otherwise predatory behavior happen in early days. As such
  /// it's no surprise that a restriction on transfers is commonplace.

  /// There's a longer conversation on alignment, and what the timescales
  /// that are envisaged. In different documents (Actika Report), theres strong
  /// consensus that 0L's timescale is much longer than other networks launched
  /// in similar time: it will takes us longer to hit our stride, but we
  /// will plan to be around for much longer. Long unlocks by the people who
  /// did most work (to date the authors are unaware of any investors in the
  /// protocol) force a conversation on long-term fundamentals.

  /// Very few accounts have Slow Wallet status. Slow Wallets were deployed
  /// initially only for validator account rewards. And as the network
  /// matured more uses were found for it. Notably the third-party
  /// Community Wallets enforce their disbursements only to SlowWallet accounts.
  /// The mechanism is simple: Every wallet gets the same flat amount
  /// released from a slow wallet every day. Read: it is not a percentage which gets
  ///  unlocked every day, as you may see elsewhere with "vesting" schedules.
  /// Alice, Bob, and Carol, get the same X amount every day.

  /// It's important to note that Slow Wallet restrictions only apply to one behavior: withdrawing the asset: a slow wallet's coins always exist in the
  /// user's account, and an unlimited amount can be used for its purpose: smart
  /// contract GAS fees. This is significant for tax planning in many places. We
  /// can't say if a change to Drip constant may or may not impact this, but it
  /// needs to be considered.

  /// The rate at which the slow wallets "drip" the coins, is a major economic consideration.
  /// The correct number must achieve the goals:
  /// - creates a level playing field while network is immature
  /// - predictability of supply
  /// - attractiveness to infrastructure providers
  /// - usefulness to community wallets
  /// - value to recipients

  /// The rule which emerged in 0L [TODO: sources] from earliest test networks
  /// was the principle that the largest accounts wait longest, though it
  /// seems unreasonable for the largest accounts to wait longer than 10 years.
  /// At the start of the network "largest accounts" were not very diverse,
  /// the validators which consistently operated had roughly the same amount
  /// of coins. However as the network developed other accounts,
  /// which were never validator accounts: recipients of Community wallets.

  /// Another rule, as in Economic Principles is that it should be simple:
  /// as easy to understand and explain as possible: the same amount everyday.
  /// It should't be a function, not superlinear or other curves. Similarly,
  /// for simplicity it should be the same for all accounts. Otherwise it
  /// creates opportunities for petty gaming, which is just a mental tax on
  /// everyone.

  /// So the balancing act in determining the number is: who do we think can
  /// wait approximately 10 years, and what's the single daily value which gets
  /// us to that number. As may be expected the distributions of accounts follow
  /// a power law distribution.

  /// What Was Decided for V7
  /// The decision was simply to scale proportionately the
  /// existing drip. After some analysis it seemed like
  /// a) the coin split would be approximately 35X
  /// b) that the canonical goal of the epoch drip is to have
  /// all the largest major accounts completely unlock after 10 years
  /// and this was in the 30,000 to 50,000 daily depoch drip.
  /// Thus the the V5 epoch drip of 1,000 daily would simply scale
  /// up, and then that would be within the acceptable range for the
  /// community goals.

  /// Naive Changes
  /// There are some changes which are tempting, but would ultimately
  /// be unstable. And the main one is: making changes because of the local
  /// macro-economic environment the users are currently in.
  /// Changes here should likely only be made if the game has failed, or
  /// about to fail, BECAUSE THIS NUMBER IS WRONG. The game may be unstable for
  /// a number of reasons (e.g. attract new participants). Will changing this
  /// parameter bring it back to stability?
  /// In general and this is a refrain in Economic Principles, Proof of Fee,
  /// is that obsessing over your mechanism (auction), has diminishing returns
  /// over increasing the number of participants (bidders). If the network
  /// is failing to attract new bidders, it is not the auction is failing
  /// (slow wallet is a type of auction in a way), but that the product
  /// is not drawing enough people.
  /// Principles of design aside, if the Slow Wallet drip gets adjusted
  /// based on a localized need, the environment will likely change again, and
  /// thus beg for another adjustment. Ultimately, there can be no progress if
  /// there is no ability to plan (you can intuit this this from your
  /// experience with  fiat currencies with both price inflation and rate
  /// hikes). In practice, the repeated negotiation in the community will be
  /// exhausting, and is exactly what autonomous policy aims to prevent.

  /// Take a specific, frequent, request: "let's increase the slow wallet
  /// drip from A to B because Alice, expects to use $X US Dollars amount
  /// every months for whatever". Clearly, all assets are connected to external
  /// markets. The price for a digital asset might be related to its particular
  /// mechanisms, or it may not. A change in relative attractiveness of risk
  /// assets due to macro policy may be more strongly correlated.
  /// And if the policies do matter most, then the causality is not obvious.
  /// Assuming the same levels of movement of the asset, increasing the
  /// drip from A to B because of Alice, may make it such that the price
  /// is back to the level of A because the expectation are all the same,
  /// there are just more units to fulfill those expectations. So it is
  /// reasonable to think that in the markets, the value could be the
  /// same (or more likely lower, since there's the knock-on effect of
  /// eroding predictability).

  /// Which brings us back to an earlier statement, this request assumed
  /// problem for the network and the game is somehow failing because of
  /// this rate. If nothing changes except for this value, you should
  /// expect nothing to change. The way to get Alice the outcome she hopes
  /// for, is for the community do the real work: focus on the product
  /// and make this the most loved coin.

  struct SacredCow<phantom T> has key {
    value: u64,
  }

  struct SlowDrip {}

  // only called by genesis which can spoof a signer for 0x2.
  public(friend) fun init(zero_x_two_sig: &signer) {
    chain_status::assert_genesis();
    let addr = signer::address_of(zero_x_two_sig);
    assert!( addr == @0x2, error::invalid_state(ENOT_OX2));
    bless_this_cow<SlowDrip>(zero_x_two_sig, SLOW_WALLET_EPOCH_DRIP);
  }

  /// the assert which halts the chain if the values do not match
  /// NOTE: this is the weak link: a developer can just remove one line below to get the change to go undetected in Move.
  /// This is why there are checks external to the VM, see above.
  fun assert_same(stored: u64, code: u64) {
    assert!(stored == code, error::invalid_state(EHANDS_OFF_MY_COW));
  }

  /// initialize the state
  /// DEVS: this should not be a public function. It should only be callable from genesis
  fun bless_this_cow<T>(zero_x_two_sig: &signer, value: u64) {
    chain_status::assert_genesis();

    let addr = signer::address_of(zero_x_two_sig);
    assert!( addr == @0x2, error::invalid_state(ENOT_OX2));
    assert!(!exists<SacredCow<T>>(addr), error::invalid_state(EALREADY_INITIALIZED));
    move_to(zero_x_two_sig, SacredCow<T> {
      value
    })
  }

  // get the stored value
  public fun get_stored<T>(): u64 acquires SacredCow {
    if (!exists<SacredCow<T>>(@0x2)) return 0;
    let stored = borrow_global_mut<SacredCow<T>>(@0x2);
    stored.value
  }

  #[view]
  public fun get_slow_drip_const(): u64 acquires SacredCow {
    let stored = get_stored<SlowDrip>();
    assert_same(stored, SLOW_WALLET_EPOCH_DRIP);
    SLOW_WALLET_EPOCH_DRIP
  }
}
