
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_musical_chairs {

    use ol_framework::musical_chairs;
    use ol_framework::mock;
    // use aptos_std::debug::print;
    use std::vector;
    use std::fixed_point32;


    #[test(vm = @ol_framework)]
    public entry fun eval_compliance_happy(vm: signer) {

      let vals = mock::genesis_n_vals(5);
      assert!(vector::length(&vals) == 5, 7357001);

            // all vals compliant
      mock::mock_all_vals_good_performance(&vm);

      let (good, bad, ratio) = musical_chairs::eval_compliance();
      assert!(vector::length(&good) == 5, 7357002);
      assert!(vector::length(&bad) == 0, 7357003);
      assert!(fixed_point32::is_zero(ratio), 7357004);


      let (_outgoing_compliant_set, _new_set_size) = musical_chairs::stop_the_music(&vm);

    }

    #[test(vm = @ol_framework)]
    // only one seat opens up
    public entry fun eval_compliance_one_val(vm: signer) {

      let vals = mock::genesis_n_vals(5);
      assert!(vector::length(&vals) == 5, 7357001);

      // all vals compliant
      mock::mock_case_1(&vm, *vector::borrow(&vals, 0));

      let (good, bad, bad_ratio) = musical_chairs::eval_compliance();
      assert!(vector::length(&good) == 1, 7357002);
      assert!(vector::length(&bad) == 4, 7357003);
      assert!(!fixed_point32::is_zero(bad_ratio), 7357004);
      assert!(fixed_point32::create_from_rational(4, 5) == bad_ratio, 7357005);


      let (_outgoing_compliant_set, _new_set_size) = musical_chairs::stop_the_music(&vm);

    }
}