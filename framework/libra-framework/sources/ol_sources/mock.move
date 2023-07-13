// Some fixtures are complex and are repeatedly needed
#[test_only]
module ol_framework::mock {
  #[test_only]
  use aptos_framework::stake;
  #[test_only]
  use aptos_framework::reconfiguration;
  #[test_only]
  use ol_framework::cases;
  #[test_only]
  use ol_framework::vouch;
  #[test_only]
  use std::vector;
  #[test_only]
  use aptos_framework::genesis;
  #[test_only]
  use aptos_framework::account;
  #[test_only]
  use ol_framework::slow_wallet;
  #[test_only]
  use ol_framework::proof_of_fee;
  #[test_only]
  use ol_framework::validator_universe;
  #[test_only]
  use aptos_framework::timestamp;
  #[test_only]
  use aptos_framework::system_addresses;
  #[test_only]
  use ol_framework::epoch_boundary;
  #[test_only]
  use aptos_framework::coin;
  #[test_only]
  use ol_framework::gas_coin::{Self, GasCoin};
  #[test_only]
  use aptos_framework::transaction_fee;
  // #[test_only]
  // use aptos_std::debug::print;

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
      stake::mock_performance(vm, addr, 1, 0);
      assert!(cases::get_case(addr) == 1, 777703);
    }


    #[test_only]
    // did not do enough mining, but did validate.
    public fun mock_case_4(vm: &signer, addr: address){
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, 0, 100); // 100 failing proposals

      assert!(cases::get_case(addr) == 4, 777703);
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

    //////// PROOF OF FEE ////////
    #[test_only]
    public fun pof_default(): (vector<address>, vector<u64>, vector<u64>){

      // system_addresses::assert_ol(vm);
      let vals = stake::get_current_validators();

      let (bids, expiry) = mock_bids(&vals);

      // DiemAccount::slow_wallet_epoch_drip(vm, 100000); // unlock some coins for the validators

      // make all validators pay auction fee
      // the clearing price in the fibonacci sequence is is 1
      let (alice_bid, _) = proof_of_fee::current_bid(*vector::borrow(&vals, 0));
      assert!(alice_bid == 1, 777703);
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
        proof_of_fee::set_bid(&sig, b, 1000);
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
    }

    #[test_only]
    public fun ol_initialize_coin(root: &signer) {
      system_addresses::assert_ol(root);

      let mint_cap = init_coin_impl(root);

      coin::destroy_mint_cap(mint_cap);
    }

    #[test_only]
    public fun ol_initialize_coin_and_fund_vals(root: &signer, amount: u64) {
      system_addresses::assert_ol(root);

      let mint_cap = init_coin_impl(root);

      let vals = stake::get_current_validators();
      let i = 0;

      while (i < vector::length(&vals)) {
        let addr = vector::borrow(&vals, i);
        let c = coin::mint(amount, &mint_cap);
        coin::deposit<GasCoin>(*addr, c);
        i = i + 1;
      };

      slow_wallet::slow_wallet_epoch_drip(root, amount);
      coin::destroy_mint_cap(mint_cap);
    }

    #[test_only]
    fun init_coin_impl(root: &signer): coin::MintCapability<GasCoin> {
      system_addresses::assert_ol(root);

      let (burn_cap, mint_cap) = gas_coin::initialize_for_test_without_aggregator_factory(root);
      coin::destroy_burn_cap(burn_cap);


      transaction_fee::initialize_fee_collection_and_distribution(root, 0);

      let initial_fees = 1000000 * 100;
      let tx_fees = coin::mint(initial_fees, &mint_cap);
      transaction_fee::pay_fee(root, tx_fees);

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
      let framework_sig = account::create_signer_for_test(@aptos_framework);
      ol_test_genesis(&framework_sig);

      let val_addr = personas();
      let i = 0;
      while (i < num) {
        let val = vector::borrow(&val_addr, i);
        let sig = account::create_signer_for_test(*val);

        let (_sk, pk, pop) = stake::generate_identity();
        // stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);
        validator_universe::test_register_validator(root, &pk, &pop, &sig, 100, true, true);

        vouch::init(&sig);
        vouch::test_set_buddies(*val, val_addr);

        // TODO: validators should have a balance
        // in Mock, we should use the same validator creation path as genesis.move
        let _b = coin::balance<GasCoin>(*val);

        i = i + 1;
      };


      genesis::test_end_genesis(&framework_sig);

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
        epoch_boundary::ol_reconfigure_for_test(root);
        timestamp::fast_forward_seconds(EPOCH_DURATION);
        reconfiguration::reconfigure_for_test_custom();
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
    ol_initialize_coin(&root);
    let epoch = reconfiguration::current_epoch();
    trigger_epoch(&root);
    let new_epoch = reconfiguration::current_epoch();
    assert!(new_epoch > epoch, 7357001);
  }

  #[test(root = @ol_framework)]
  public entry fun meta_val_perf(root: signer) {
    // genesis();

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


}
