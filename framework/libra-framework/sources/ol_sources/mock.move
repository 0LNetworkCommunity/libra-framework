// Some fixtures are complex and are repeatedly needed
#[test_only]
module ol_framework::mock {
  use diem_framework::stake;
  use diem_framework::reconfiguration;
  use ol_framework::grade;
  use ol_framework::vouch;
  use std::vector;
  use diem_framework::genesis;
  use diem_framework::account;
  use ol_framework::slow_wallet;
  use ol_framework::proof_of_fee;
  use ol_framework::validator_universe;
  use diem_framework::timestamp;
  use diem_framework::system_addresses;
  use ol_framework::epoch_boundary;
  use diem_framework::coin;
  use ol_framework::libra_coin::{Self, LibraCoin};
  use diem_framework::transaction_fee;
  use ol_framework::ol_account;
  use ol_framework::tower_state;
  use ol_framework::vdf_fixtures;
  use ol_framework::epoch_helper;
  use ol_framework::musical_chairs;
  use ol_framework::globals;
  use diem_framework::block;

  // use diem_std::debug::print;

  const ENO_GENESIS_END_MARKER: u64 = 1;
  const EDID_NOT_ADVANCE_EPOCH: u64 = 2;
  /// coin supply does not match expected
  const ESUPPLY_MISMATCH: u64 = 3;

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
  public fun mock_case_1(vm: &signer, addr: address){
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, 100, 10);
      let (compliant, _, _, _) = grade::get_validator_grade(addr, 100);
      assert!(compliant, 01);
    }


    #[test_only]
    // did not do enough mining, but did validate.
    public fun mock_case_4(vm: &signer, addr: address){
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, 0, 100); // 100 failing proposals
      let (compliant, _, _, _) = grade::get_validator_grade(addr, 0);
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

    //////// TOWER ///////
    #[test_only]
    public fun tower_default(root: &signer) {
      let vals = stake::get_current_validators();
      tower_state::set_difficulty(root, globals::get_vdf_difficulty_baseline(), globals::get_vdf_security_baseline()); // original fixtures pre-wesolowski change.
      let i = 0;
      while (i < vector::length(&vals)) {

        let addr = vector::borrow(&vals, i);
        tower_state::test_helper_init_val(
            &account::create_signer_for_test(*addr),
            vdf_fixtures::alice_0_easy_chal(),
            vdf_fixtures::alice_0_easy_sol(),
            vdf_fixtures::easy_difficulty(),
            vdf_fixtures::security(),
        );
        i = i + 1;
      };
    }

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

    use diem_framework::chain_status;
    #[test_only]
    public fun ol_test_genesis(root: &signer) {
      system_addresses::assert_ol(root);
      genesis::setup();
      genesis::test_end_genesis(root);

      let mint_cap = init_coin_impl(root);
      libra_coin::restore_mint_cap(root, mint_cap);

      assert!(!chain_status::is_genesis(), 0);
    }

    #[test_only]
    public fun ol_initialize_coin(root: &signer) {
      system_addresses::assert_ol(root);

      let mint_cap = init_coin_impl(root);

      libra_coin::restore_mint_cap(root, mint_cap);
    }

    #[test_only]
    public fun ol_initialize_coin_and_fund_vals(root: &signer, amount: u64,
    drip: bool) {
      system_addresses::assert_ol(root);


      let mint_cap = if (coin::is_coin_initialized<LibraCoin>()) {
         libra_coin::extract_mint_cap(root)
      } else {
        init_coin_impl(root)
      };

      let vals = stake::get_current_validators();
      let i = 0;

      while (i < vector::length(&vals)) {
        let addr = vector::borrow(&vals, i);
        let c = coin::test_mint(amount, &mint_cap);
        ol_account::deposit_coins(*addr, c);

        let b = coin::balance<LibraCoin>(*addr);
        assert!(b == amount, 0001);


        i = i + 1;
      };

      if (drip) {
        slow_wallet::slow_wallet_epoch_drip(root, amount);
      };
      libra_coin::restore_mint_cap(root, mint_cap);
    }

    #[test_only]
    fun init_coin_impl(root: &signer): coin::MintCapability<LibraCoin> {
      system_addresses::assert_ol(root);

      let (burn_cap, mint_cap) = libra_coin::initialize_for_test_without_aggregator_factory(root);
      coin::destroy_burn_cap(burn_cap);

      transaction_fee::initialize_fee_collection_and_distribution(root, 0);

      let initial_fees = 1000000 * 100; // coin scaling * 100 coins
      let tx_fees = coin::test_mint(initial_fees, &mint_cap);
      transaction_fee::vm_pay_fee(root, @ol_framework, tx_fees);
      let supply_pre = libra_coin::supply();
      assert!(supply_pre == initial_fees, ESUPPLY_MISMATCH);
      libra_coin::test_set_final_supply(root, initial_fees);

      mint_cap
    }

    #[test_only]
    public fun personas(): vector<address> {
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
      val_addr
    }

    #[test_only]
    /// mock up to 6 validators alice..frank
    public fun genesis_n_vals(root: &signer, num: u64): vector<address> {
      system_addresses::assert_ol(root);
      let framework_sig = account::create_signer_for_test(@diem_framework);
      ol_test_genesis(&framework_sig);
      // need to initialize musical chairs separate from genesis.
      let musical_chairs_default_seats = 10;
      musical_chairs::initialize(root, musical_chairs_default_seats);

      let val_addr = personas();
      let i = 0;
      while (i < num) {
        let val = vector::borrow(&val_addr, i);
        let sig = account::create_signer_for_test(*val);

        let (_sk, pk, pop) = stake::generate_identity();
        // stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);
        validator_universe::test_register_validator(root, &pk, &pop, &sig, 100,
        true, true);

        vouch::init(&sig);
        vouch::test_set_buddies(*val, val_addr);

        i = i + 1;
      };

      timestamp::fast_forward_seconds(2); // or else reconfigure wont happen
      stake::test_reconfigure(root, validator_universe::get_eligible_validators());

      stake::get_current_validators()
    }

    #[test_only]
    const EPOCH_DURATION: u64 = 60;

    #[test_only]
    // NOTE: The order of these is very important.
    // ol first runs its own accounting at end of epoch with epoch_boundary
    // Then the stake module needs to update the validators.
    // the reconfiguration module must run last, since no other
    // transactions or operations can happen after the reconfig.
    public fun trigger_epoch(root: &signer) {
        let old_epoch = reconfiguration::get_current_epoch();
        timestamp::fast_forward_seconds(EPOCH_DURATION);

        epoch_boundary::ol_reconfigure_for_test(root, old_epoch,
        block::get_current_block_height());

        // always advance
        assert!(reconfiguration::get_current_epoch() > old_epoch,
        EDID_NOT_ADVANCE_EPOCH);

        // epoch helper should always be in sync
        assert!(reconfiguration::get_current_epoch() ==
        epoch_helper::get_current_epoch(), 666);
    }

  //   // function to deposit into network fee account
  //   public fun mock_network_fees(vm: &signer, amount: u64) {
  //     Testnet::assert_testnet(vm);
  //     let c = Diem::mint<GAS>(vm, amount);
  //     let c_value = Diem::value(&c);
  //     assert!(c_value == amount, 777707);
  //     TransactionFee::pay_fee(c);
  //   }


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
    let genesis_mint = 1000000;
    ol_initialize_coin_and_fund_vals(root, genesis_mint, true);
    let supply_pre = libra_coin::supply();
    let mocked_tx_fees = 1000000 * 100;
    assert!(supply_pre == mocked_tx_fees + (n_vals * genesis_mint), 73570001);
  }


  #[test(root = @ol_framework)]
  fun test_setting_pof(root: &signer) {
    // Scenario: unit testing that pof_default results in a usable auction
    let n_vals = 5;
    let vals = genesis_n_vals(root, n_vals); // need to include eve to init funds
    pof_default();

    proof_of_fee::fill_seats_and_get_price(root, n_vals, &vals, &vals);

    let (nominal_reward, entry_fee, clearing_percent, median_bid ) = proof_of_fee::get_consensus_reward();

    assert!(nominal_reward == 1000000, 73570001);
    assert!(clearing_percent == 1, 73570002);
    assert!(entry_fee == 999, 73570003);
    assert!(median_bid == 3, 73570004);
  }
}
