// Some fixutres are complex and are repeatedly needed
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
  // #[test_only]
  // use aptos_std::debug::print;
  #[test_only]
  use ol_framework::proof_of_fee;
  // #[test_only]
  // use aptos_framework::system_addresses;


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
      assert!(cases::get_case(addr, 0, 15) == 1, 777703);
    }


    #[test_only]
    // did not do enough mining, but did validate.
    public fun mock_case_4(vm: &signer, addr: address){
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, 0, 100); // 100 failing proposals

      assert!(cases::get_case(addr, 0, 15) == 4, 777703);
    }

    // Mock all nodes being compliant case 1
    #[test_only]
    public fun mock_all_vals_good_performance(vm: &signer) {

      let vals =  stake::get_current_validators();

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
    public fun genesis() {
      genesis::setup();
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
      val_addr
    }

    #[test_only]
    /// mock up to 6 validators alice..frank
    public fun genesis_n_vals(num: u64): vector<address> {
      genesis::setup();

      let val_addr = personas();
      let i = 0;
      while (i < num) {

        let val = vector::borrow(&val_addr, i);
        let sig = account::create_signer_for_test(*val);
        let (_sk, pk, pop) = stake::generate_identity();
        stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);
        vouch::init(&sig);
        vouch::test_set_buddies(*val, val_addr);
        i = i + 1;
      };

      let framework_sig = account::create_signer_for_test(@aptos_framework);
      genesis::test_end_genesis(&framework_sig);

      stake::get_current_validators()
    }

    #[test_only]
    public fun trigger_epoch() {
        stake::end_epoch(); // additionally forwards EPOCH_DURATION seconds
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
  #[test]
  /// test we can trigger an epoch reconfiguration.
  public fun meta_epoch() {
    genesis();
    let epoch = reconfiguration::current_epoch();
    trigger_epoch();
    let new_epoch = reconfiguration::current_epoch();
    assert!(new_epoch > epoch, 7357001);

  }

  #[test(vm = @ol_framework)]
  public entry fun meta_val_perf(vm: signer) {
    // genesis();
    
    let set = genesis_n_vals(4);
    assert!(vector::length(&set) == 4, 7357001);

    let addr = vector::borrow(&set, 0);

    // will assert! case_1
    mock_case_1(&vm, *addr);


    pof_default();

    // will assert! case_4
    mock_case_4(&vm, *addr);

    reset_val_perf_all(&vm);

    mock_all_vals_good_performance(&vm);
  }


}
