// Some fixtures are complex and are repeatedly needed
#[test_only]
module ol_framework::mock {
  use std::signer;
  use std::vector;
  use diem_framework::chain_status;
  use diem_framework::coin;
  use diem_framework::chain_id;
  use diem_framework::block;
  use diem_framework::stake;
  use diem_framework::account;
  use diem_framework::genesis;
  use diem_framework::timestamp;
  use diem_framework::reconfiguration;
  use diem_framework::system_addresses;
  use diem_framework::transaction_fee;
  use diem_std::from_bcs;
  use std::bcs;
  use ol_framework::activity;
  use ol_framework::ancestry;
  use ol_framework::filo_migration;
  use ol_framework::grade;
  use ol_framework::vouch;
  use ol_framework::slow_wallet;
  use ol_framework::proof_of_fee;
  use ol_framework::validator_universe;
  use ol_framework::epoch_boundary;
  use ol_framework::libra_coin::{Self, LibraCoin};
  use ol_framework::ol_account;
  use ol_framework::epoch_helper;
  use ol_framework::musical_chairs;
  use ol_framework::infra_escrow;
  use ol_framework::vouch_limits;
  use ol_framework::root_of_trust;
  use ol_framework::page_rank_lazy;

  use diem_std::debug::print;

  const ENO_GENESIS_END_MARKER: u64 = 1;
  const EDID_NOT_ADVANCE_EPOCH: u64 = 2;
  /// coin supply does not match expected
  const ESUPPLY_MISMATCH: u64 = 3;
  const EPOCH_DURATION: u64 = 60;

  #[test_only]
  public fun reset_val_perf_one(vm: &signer, addr: address) {
    stake::mock_performance(vm, addr, 0, 0);
  }

  #[test_only]
  public fun reset_val_perf_all(vm: &signer) {
    let vals = stake::get_current_validators();
    let i = 0;
    while (i < vector::length(&vals)) {
      let a = vector::borrow(&vals, i);
      stake::mock_performance(vm, *a, 0, 0);
      i = i + 1;
    };
  }


  #[test_only]
  public fun mock_case_1(vm: &signer, addr: address) {
    assert!(stake::is_valid(addr), 01);
    stake::mock_performance(vm, addr, 100, 10);
    let (compliant, _, _) = grade::get_validator_grade(addr);
    assert!(compliant, 01);
  }


  #[test_only]
  // did not do enough mining, but did validate.
  public fun mock_case_4(vm: &signer, addr: address) {
    assert!(stake::is_valid(addr), 01);
    stake::mock_performance(vm, addr, 0, 100); // 100 failing proposals
    // let (compliant, _, _, _) = grade::get_validator_grade(addr, 0);
    let (compliant, _, _) = grade::get_validator_grade(addr);
    assert!(!compliant, 02);
  }

  // Mock all nodes being compliant case 1
  #[test_only]
  public fun mock_all_vals_good_performance(vm: &signer) {
    let vals = stake::get_current_validators();
    let i = 0;
    while (i < vector::length(&vals)) {
      let a = vector::borrow(&vals, i);
      mock_case_1(vm, *a);
      i = i + 1;
    };
  }

  // Mock some nodes not compliant
  /*#[test_only]
  public fun mock_some_vals_bad_performance(vm: &signer, bad_vals: u64) {
    let vals = stake::get_current_validators();
    let vals_compliant = vector::length(&vals) - bad_vals;
    let i = 0;
    while (i < vals_compliant) {
      let a = vector::borrow(&vals, i);
      stake::mock_performance(vm, *a, 0, 100); // 100 failing proposals
      i = i + 1;
    };
  }*/

  //////// PROOF OF FEE ////////
  #[test_only]
  public fun pof_default(): (vector<address>, vector<u64>, vector<u64>){

    let vals = stake::get_current_validators();

    let (bids, expiry) = mock_bids(&vals);

    // make all validators pay auction fee
    // the clearing price in the fibonacci sequence is is 1
    let (alice_bid, _) = proof_of_fee::current_bid(*vector::borrow(&vals, 0));
    assert!(alice_bid == 1, 03);
    (vals, bids, expiry)
  }

  #[test_only]
  public fun mock_bids(vals: &vector<address>): (vector<u64>, vector<u64>) {
    // system_addresses::assert_ol(vm);
    let bids = vector::empty<u64>();
    let expiry = vector::empty<u64>();
    let i = 0;
    let prev = 0;
    let fib = 1;
    while (i < vector::length(vals)) {

      vector::push_back(&mut expiry, 1000);
      let b = prev + fib;
      vector::push_back(&mut bids, b);

      let a = vector::borrow(vals, i);
      let sig = account::create_signer_for_test(*a);
      // initialize and set.
      proof_of_fee::pof_update_bid(&sig, b, 1000);
      prev = fib;
      fib = b;
      i = i + 1;
    };

    (bids, expiry)
  }


  #[test_only]
  public fun ol_test_genesis(root: &signer) {
    system_addresses::assert_ol(root);
    genesis::setup();
    genesis::test_end_genesis(root);

    let mint_cap = coin_init_minimal(root);
    libra_coin::restore_mint_cap(root, mint_cap);

    assert!(!chain_status::is_genesis(), 0);
  }

  #[test_only]
  public fun ol_initialize_coin(root: &signer) {
    system_addresses::assert_ol(root);

    let mint_cap = coin_init_minimal(root);

    libra_coin::restore_mint_cap(root, mint_cap);
  }

  #[test_only]
  public fun ol_mint_to(root: &signer, addr: address, amount: u64) {
    system_addresses::assert_ol(root);

    let mint_cap = if (coin::is_coin_initialized<LibraCoin>()) {
      libra_coin::extract_mint_cap(root)
    } else {
      coin_init_minimal(root)

    };

    if (!account::exists_at(addr)) {
        ol_account::create_account(root, addr);
    };

    let c = coin::test_mint(amount, &mint_cap);
    ol_account::deposit_coins(addr, c);

    let b = coin::balance<LibraCoin>(addr);
    assert!(b == amount, 0001);

    libra_coin::restore_mint_cap(root, mint_cap);
  }

  #[test_only]
  public fun ol_initialize_coin_and_fund_vals(root: &signer, amount: u64,
  drip: bool) {
    system_addresses::assert_ol(root);

    let mint_cap = if (coin::is_coin_initialized<LibraCoin>()) {
      libra_coin::extract_mint_cap(root)
    } else {
      coin_init_minimal(root)
    };

    let vals = stake::get_current_validators();
    let i = 0;
    while (i < vector::length(&vals)) {
      let addr = vector::borrow(&vals, i);
      let c = coin::test_mint(amount, &mint_cap);
      ol_account::deposit_coins(*addr, c);

      let b = libra_coin::balance(*addr);
      assert!(b == amount, 0001);

      i = i + 1;
    };

    if (drip) {
      slow_wallet::slow_wallet_epoch_drip(root, amount);
    };
    libra_coin::restore_mint_cap(root, mint_cap);
  }

  #[test_only]
  // For unit test, we need to try to initialize the minimal state for
  // user transactions. In the case of a unit tests which does a genesis with validators, this will not attempt to re-initialize the state.
  fun coin_init_minimal(root: &signer): coin::MintCapability<LibraCoin> {
    system_addresses::assert_ol(root);

    let (burn_cap, mint_cap) = libra_coin::initialize_for_test(root);
    coin::destroy_burn_cap(burn_cap);

    transaction_fee::initialize_fee_collection_and_distribution(root, 0);

    let initial_fees = 5_000_000 * 100; // coin scaling * 100 coins
    let tx_fees = coin::test_mint(initial_fees, &mint_cap);
    transaction_fee::vm_pay_fee(root, @ol_framework, tx_fees);

    // OL Network has no central treasury
    // the infra escrow came from prior miner
    // contributions

    // Forge Bruce
    let fortune = 100_000_000_000;
    let bruce_address = @0xBA7;
    ol_account::create_account(root, bruce_address);

    // Bruce mints a fortune
    let bruce = account::create_signer_for_test(bruce_address);
    let fortune_mint = coin::test_mint(fortune, &mint_cap);

    ol_account::deposit_coins(bruce_address, fortune_mint);
    // Bruce funds infra escrow
    infra_escrow::init_escrow_with_deposit(root, &bruce, 37_000_000_000);

    let supply_pre = libra_coin::supply();
    assert!(supply_pre == (initial_fees + fortune), ESUPPLY_MISMATCH);
    libra_coin::test_set_final_supply(root, initial_fees);

    mint_cap
  }

  #[test_only]
  public fun personas(count: u64): vector<address> {
    assert!(count > 0, 735701);
    assert!(count < 11, 735702);

    let val_addr = vector::empty<address>();
    vector::push_back(&mut val_addr, @0x1000a);
    vector::push_back(&mut val_addr, @0x1000b);
    vector::push_back(&mut val_addr, @0x1000c);
    vector::push_back(&mut val_addr, @0x1000d);
    vector::push_back(&mut val_addr, @0x1000e);
    vector::push_back(&mut val_addr, @0x1000f);
    vector::push_back(&mut val_addr, @0x10010); // g
    vector::push_back(&mut val_addr, @0x10011); // h
    vector::push_back(&mut val_addr, @0x10012); // i
    vector::push_back(&mut val_addr, @0x10013); // k

    vector::trim(&mut val_addr, count);

    val_addr
  }

  #[test_only]
  /// mock up to 6 validators alice..frank
  public fun genesis_n_vals(root: &signer, num: u64): vector<address> {
    // create vals with vouches
    system_addresses::assert_ol(root);
    let framework_sig = account::create_signer_for_test(@diem_framework);
    ol_test_genesis(&framework_sig);

    create_validator_accounts(root, num, true);

    // timestamp::fast_forward_seconds(2); // or else reconfigure wont happen
    stake::test_reconfigure(root, validator_universe::get_eligible_validators());

    stake::get_current_validators()
  }

  #[test_only]
  // Helper function to create a bunch of test signers
  public fun create_test_end_users(framework: &signer, count: u64, start_idx: u64): vector<signer> {
    system_addresses::assert_diem_framework(framework);

    let signers = vector::empty<signer>();
    let i = 0;
    while (i < count) {
      let sig = create_user_from_u64(framework, start_idx + i);
      vector::push_back(&mut signers, sig);
      i = i + 1;
    };
    signers
  }

  #[test_only]
  // Create signer from u64, good for creating many accounts
  public fun create_user_from_u64(framework: &signer, idx: u64): signer {
    system_addresses::assert_diem_framework(framework);

    let idx = 0xA + idx;
    // Create a vector of bytes to represent the address
    // Convert u64 to bytes first using BCS serialization
    let addr_bytes = bcs::to_bytes(&idx);

    // When idx is small, the BCS representation will be short
    // Pad with zeros if needed to get a valid address (32 bytes for Move addresses)
    while (vector::length(&addr_bytes) < 32) {
      vector::push_back(&mut addr_bytes, 0);
    };

    // Convert bytes to address using from_bcs module
    let addr_as_addr = from_bcs::to_address(addr_bytes);

    ol_account::create_account(framework, addr_as_addr);

    // Create test signer from the generated address
    account::create_signer_for_test(addr_as_addr)
  }

  #[test_only]
  // Helper to extract addresses from signers
  public fun collect_addresses(signers: &vector<signer>): vector<address> {
    let addresses = vector::empty<address>();
    let i = 0;
    let len = vector::length(signers);
    while (i < len) {
      let addr = signer::address_of(vector::borrow(signers, i));
      vector::push_back(&mut addresses, addr);
      i = i + 1;
    };
    addresses
  }

  #[test_only]
  public fun create_validator_accounts(root: &signer, num: u64, with_vouches: bool) {

    // need to initialize musical chairs separate from genesis.
    let musical_chairs_default_seats = 10;
    musical_chairs::initialize(root, musical_chairs_default_seats);

    let val_addr = personas(num);

    root_of_trust::framework_migration(root, val_addr, vector::length(&val_addr), 10000);

    let i = 0;
    while (i < num) {
      let val = vector::borrow(&val_addr, i);
      let sig = account::create_signer_for_test(*val);
      let (_sk, pk, pop) = stake::generate_identity();
      let should_end_epoch = false;
      validator_universe::test_register_validator(root, &pk, &pop, &sig, 100, true, should_end_epoch);

      vouch::init(&sig);
      page_rank_lazy::maybe_initialize_trust_record(&sig);

      i = i + 1;
    };

    root_of_trust::genesis_initialize(root, stake::get_current_validators());

    if (with_vouches) {
      vouch::set_vouch_price(root, 0);
      let k = 0;
      while (k < num) {
        let grantor = vector::borrow(&val_addr, k);
        let grantor_sig = account::create_signer_for_test(*grantor);
        let f = 0;
            while (f < vector::length(&val_addr)) {
              let recipient = vector::borrow(&val_addr, f);

              if (grantor != recipient) {
                vouch::vouch_for(&grantor_sig, *recipient);
              };
              f = f + 1;
            };
          k = k + 1;
        };

    }
  }


  #[test_only]
  // NOTE: The order of these is very important.
  // ol first runs its own accounting at end of epoch with epoch_boundary
  // Then the stake module needs to update the validators.
  // the reconfiguration module must run last, since no other
  // transactions or operations can happen after the reconfig.
  public fun trigger_epoch(root: &signer) {
    trigger_epoch_exactly_at(
      root,
      reconfiguration::get_current_epoch(),
      block::get_current_block_height()
    );
  }

  #[test_only]
  public fun trigger_epoch_exactly_at(root: &signer, old_epoch: u64, round: u64) {
    timestamp::fast_forward_seconds(EPOCH_DURATION);
    epoch_boundary::ol_reconfigure_for_test(root, old_epoch, round);

    // always advance
    assert!(reconfiguration::get_current_epoch() > old_epoch,
    EDID_NOT_ADVANCE_EPOCH);

    // epoch helper should always be in sync
    assert!(reconfiguration::get_current_epoch() == epoch_helper::get_current_epoch(), 666);
  }


  #[test_only]
  /// Creates a vouching network where target_account has a vouch score of 100
  /// This will create validators if they don't exist, and set up vouches
  /// @param framework - framework signer
  /// @param target_account - the account that will have a vouch score of 100
  /// @return validators - the list of validators created/used for vouching
  public fun mock_vouch_score_50(framework: &signer, target_account: address): vector<address> {
    system_addresses::assert_diem_framework(framework);

    let parent_account = @0xdeadbeef;
    ol_account::create_account(framework, parent_account);
    ol_mint_to(framework, parent_account, 10000);

    // Get the current validator set instead of creating a new genesis
    // TODO: replace for root of trust implementation
    let vals = stake::get_current_validators();

    // Do the typical onboarding process: by transfer
    let parent_sig = account::create_signer_for_test(parent_account);
    let target_signer = account::create_signer_for_test(target_account);
    ol_account::transfer(&parent_sig, target_account, 1000);
    // vouch will be missing unless the initialized it through filo_migration
    vouch::init(&target_signer);
    page_rank_lazy::maybe_initialize_trust_record(&target_signer);


    // Each validator vouches
    let i = 0;
    while (i < vector::length(&vals)) {
        let val_addr = *vector::borrow(&vals, i);
        let val_signer = account::create_signer_for_test(val_addr);

        // Initialize vouch module for validator if not already done
        vouch::vouch_for(&val_signer, target_account);

        i = i + 1;
    };

    // Verify the score is exactly 50
    // let score = vouch_metrics::calculate_total_vouch_quality(target_account);
    // assert!(score == 50, 735700);


    vals
  }


  //////// META TESTS ////////
  #[test(root=@ol_framework)]
  /// test we can trigger an epoch reconfiguration.
  public fun meta_epoch(root: signer) {
    ol_test_genesis(&root);
    musical_chairs::initialize(&root, 10);
    // ol_initialize_coin(&root);
    let epoch = reconfiguration::current_epoch();
    trigger_epoch(&root);
    let new_epoch = reconfiguration::current_epoch();
    assert!(new_epoch > epoch, 7357001);
  }

  #[test(root = @ol_framework)]
  public entry fun meta_val_perf(root: signer) {
    let set = genesis_n_vals(&root, 4);
    assert!(vector::length(&set) == 4, 7357001);

    let addr = vector::borrow(&set, 0);

    // will assert! case_1
    mock_case_1(&root, *addr);

    pof_default();

    // will assert! case_4
    mock_case_4(&root, *addr);

    reset_val_perf_all(&root);

    mock_all_vals_good_performance(&root);
  }

  #[test(root = @ol_framework)]
  fun test_initial_supply(root: &signer) {
    // Scenario:
    // There are two community wallets. Alice and Bob's
    // EVE donates to both. But mostly to Alice.
    // The Match Index, should reflect that.

    let n_vals = 5;
    let _vals = genesis_n_vals(root, n_vals); // need to include eve to init funds
    let genesis_mint = 1_000_000;
    ol_initialize_coin_and_fund_vals(root, genesis_mint, true);
    let supply_pre = libra_coin::supply();
    let bruce_fortune = 100_000_000_000;
    let mocked_tx_fees = 5_000_000 * 100;
    assert!(supply_pre == bruce_fortune + mocked_tx_fees + (n_vals * genesis_mint), 73570001);
  }


  #[test(root = @ol_framework)]
  fun test_setting_pof(root: &signer) {
    // Scenario: unit testing that pof_default results in a usable auction
    let n_vals = 5;
    let vals = genesis_n_vals(root, n_vals); // need to include eve to init funds
    pof_default();

    proof_of_fee::fill_seats_and_get_price(root, n_vals, &vals, &vals);

    let (nominal_reward, entry_fee, clearing_percent, median_bid ) = proof_of_fee::get_consensus_reward();

    assert!(nominal_reward == 1_000_000, 73570001);
    assert!(clearing_percent == 1, 73570002);
    assert!(entry_fee == 1_000, 73570003);
    assert!(median_bid == 3, 73570004);
  }


  #[test(framework = @0x1, bob = @0x10002)]
  fun meta_test_minimal_account_init(framework: &signer, bob: address) {
    timestamp::set_time_has_started_for_testing(framework);
    chain_id::initialize_for_test(framework, 4);
    ol_mint_to(framework, bob, 123);
  }

  #[test(root = @ol_framework)]
  fun test_mock_vouch_score_50(root: &signer) {
    // Set up genesis with validators first
    let _vals = genesis_n_vals(root, 4);
    ol_initialize_coin_and_fund_vals(root, 10000, true);
    // Set up a target account that should receive a vouch score of ~100
    let target_address = @0x12345;

    mock_vouch_score_50(root, target_address);

    // let score = vouch_metrics::calculate_total_vouch_quality(target_address);
    // assert!(score == 50, 735700);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  fun validator_vouches_mocked_correctly(root: &signer, alice: address) {
    // create vals without vouches
    let vals = genesis_n_vals(root, 10);

    let i = 0;
    while (i < vector::length(&vals)) {
      let val = vector::borrow(&vals, i);
      // check we have ancestry initialized
      let ancestry = ancestry::get_tree(*val);
      // all validators have ancestr
      let parent = vector::borrow(&ancestry, 0);
      assert!(parent == &@diem_framework, 7357000);
      i = i + 1;
    };

    // in testnet the genesis validators are the seed root of trust for human verification
    let is_root = root_of_trust::is_root_at_registry(@ol_framework, alice);
    assert!(is_root, 7357001);

    let (given_vouches, _ ) = vouch::get_given_vouches(alice);
    assert!(vector::length(&given_vouches) == 9, 7357002);
    let (received_vouches, _) = vouch::get_received_vouches(alice);
    assert!(vector::length(&received_vouches) == 9, 7357002);

    let score = page_rank_lazy::get_trust_score(alice);
    // excluding self, the voucher has 9 vouches of remaining validator
    // set which are also roots of trust
    assert!(score == vector::length(&received_vouches) * 50, 7357002);

    let remaining = vouch_limits::get_remaining_vouches(alice);
    print(&remaining);

  }

  #[test_only]
  /// two state initializations happen on first
  /// transaction
  public fun simulate_transaction_validation(sender: &signer) {
    let time = timestamp::now_seconds();
    // will initialize structs if first time
    activity::increment(sender, time);

    // run migrations
    // Note, Activity and Founder struct should have been set above
    filo_migration::maybe_migrate(sender);
  }


}
