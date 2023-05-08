// Some fixutres are complex and are repeatedly needed
module ol_framework::mock {
  #[test_only]
  use aptos_framework::stake;
  #[test_only]
  use aptos_framework::reconfiguration;
  #[test_only]
  use ol_framework::cases;
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
  #[test_only]
  use aptos_framework::system_addresses;

  #[test_only]
  public fun mock_case_1(vm: &signer, addr: address, success: u64, fail: u64){
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, success, fail);
      assert!(cases::get_case(vm, addr, 0, 15) == 1, 777703);
    }

  #[test(vm = @vm_reserved)]
  public entry fun test_mock_validators(vm: signer) {
    // genesis();
    
    let set = genesis_n_vals(4);

    let addr = vector::borrow(&set, 0);

    // will assert! case_1
    mock_case_1(&vm, *addr, 1, 0);

    pof_default(&vm);

    // will assert! case_4
    mock_case_4(&vm, *addr);
  }

    #[test_only]
    // did not do enough mining, but did validate.
    public fun mock_case_4(vm: &signer, addr: address){
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, 0, 100); // 100 failing proposals

      assert!(cases::get_case(vm, addr, 0, 15) == 4, 777703);
    }

    // Mock all nodes being compliant case 1
    #[test_only]
    public fun all_good_validators(vm: &signer) {

      let vals =  stake::get_current_validators();

      let i = 0;
      while (i < vector::length(&vals)) {

        let a = vector::borrow(&vals, i);
        mock_case_1(vm, *a, 0, 15);
        i = i + 1;
      };

    }

    //////// PROOF OF FEE ////////
    #[test_only]
    public fun pof_default(vm: &signer): (vector<address>, vector<u64>, vector<u64>){

      system_addresses::assert_vm(vm);
      let vals =  stake::get_current_validators();

      let (bids, expiry) = mock_bids(vm, &vals);

      // DiemAccount::slow_wallet_epoch_drip(vm, 100000); // unlock some coins for the validators

      // make all validators pay auction fee
      // the clearing price in the fibonacci sequence is is 1
      // DiemAccount::vm_multi_pay_fee(vm, &vals, 1, &b"proof of fee");

      (vals, bids, expiry)
    }

    #[test_only]
    public fun mock_bids(vm: &signer, vals: &vector<address>): (vector<u64>, vector<u64>) {
      system_addresses::assert_vm(vm);
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
    /// mock up to 6 validators alice..frank
    public fun genesis_n_vals(num: u64): vector<address> {
      genesis::setup();

      let val_addr = vector::empty<address>();

      vector::push_back(&mut val_addr, @0x1000a);
      vector::push_back(&mut val_addr, @0x1000b);
      vector::push_back(&mut val_addr, @0x1000c);
      vector::push_back(&mut val_addr, @0x1000d);
      vector::push_back(&mut val_addr, @0x1000e);
      vector::push_back(&mut val_addr, @0x1000f);

      let i = 0;
      while (i < (num -1)) {
        // create_signer_for_test
        let val = vector::borrow(&val_addr, i);
        let sig = account::create_signer_for_test(*val);
        let (_sk, pk, pop) = stake::generate_identity();
        stake::initialize_test_validator(&pk, &pop, &sig, 100, true, true);
        i = i + 1;
      };

      stake::get_current_validators()
    }

    #[test_only]
    public fun end_epoch() {
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

}
