// Some fixutres are complex and are repeatedly needed
module ol_framework::mock {
  // use DiemFramework::TowerState;
  // use Std::Vector;
  // use DiemFramework::Stats;
  // use DiemFramework::Cases;
  // // use DiemFramework::Debug::print;
  // use DiemFramework::Testnet;
  // use DiemFramework::ValidatorUniverse;
  // use DiemFramework::DiemAccount;
  // use DiemFramework::ProofOfFee;
  // use DiemFramework::DiemSystem;
  // use DiemFramework::Diem;
  // use DiemFramework::TransactionFee;
  // use DiemFramework::GAS::GAS;

  #[test_only]
  use aptos_framework::stake;
  #[test_only]
  use ol_framework::cases;
  #[test_only]
  use ol_framework::testnet;
  #[test_only]
  use std::vector;
  #[test_only]
  use aptos_framework::genesis;
  #[test_only]
  use aptos_std::debug::print;

  #[test_only]
  public fun mock_case_1(vm: &signer, addr: address, success: u64, fail: u64){
      // test::assert_testnet(vm);
      assert!(stake::is_valid(addr), 01);
      stake::mock_performance(vm, addr, success, fail);

      assert!(cases::get_case(vm, addr, 0, 15) == 1, 777703);
    }

  #[test(vm = @vm_reserved, validator = @0x123)]
  public entry fun test_mock_validators(vm: signer, validator: signer) {
    genesis::setup();
    let (_sk, pk, pop) = stake::generate_identity();
    stake::initialize_test_validator(&pk, &pop, &validator, 100, true, true);

    let vals = stake::get_current_validators();
    print(&vals);
    assert!(vector::length(&vals) > 0, 01);
    let val = vector::borrow(&vals, 0);

    // will assert! case_1
    mock_case_1(&vm, *val, 1, 0);

    // will assert! case_4
    mock_case_4(&vm, *val);
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
      testnet::assert_testnet(vm);

      let vals =  stake::get_current_validators();

      let i = 0;
      while (i < vector::length(&vals)) {

        let a = vector::borrow(&vals, i);
        mock_case_1(vm, *a, 0, 15);
        i = i + 1;
      };

    }

  //   //////// PROOF OF FEE ////////
  //   public fun pof_default(vm: &signer): (vector<address>, vector<u64>, vector<u64>){

  //     Testnet::assert_testnet(vm);
  //     let vals = ValidatorUniverse::get_eligible_validators();

  //     let (bids, expiry) = mock_bids(vm, &vals);

  //     DiemAccount::slow_wallet_epoch_drip(vm, 100000); // unlock some coins for the validators

  //     // make all validators pay auction fee
  //     // the clearing price in the fibonacci sequence is is 1
  //     DiemAccount::vm_multi_pay_fee(vm, &vals, 1, &b"proof of fee");

  //     (vals, bids, expiry)
  //   }

  //   public fun mock_bids(vm: &signer, vals: &vector<address>): (vector<u64>, vector<u64>) {
  //     Testnet::assert_testnet(vm);

  //     let bids = Vector::empty<u64>();
  //     let expiry = Vector::empty<u64>();
  //     let i = 0;
  //     let prev = 0;
  //     let fib = 1;
  //     while (i < Vector::length(vals)) {

  //       Vector::push_back(&mut expiry, 1000);
  //       let b = prev + fib;
  //       Vector::push_back(&mut bids, b);

  //       let a = Vector::borrow(vals, i);
  //       let sig = DiemAccount::scary_create_signer_for_migrations(vm, *a);
  //       // initialize and set.
  //       ProofOfFee::set_bid(&sig, b, 1000);
  //       prev = fib;
  //       fib = b;
  //       i = i + 1;
  //     };

  //     (bids, expiry)

  //   }

  //   // function to deposit into network fee account
  //   public fun mock_network_fees(vm: &signer, amount: u64) {
  //     Testnet::assert_testnet(vm);
  //     let c = Diem::mint<GAS>(vm, amount);
  //     let c_value = Diem::value(&c);
  //     assert!(c_value == amount, 777707);
  //     TransactionFee::pay_fee(c);
  //   }

}
