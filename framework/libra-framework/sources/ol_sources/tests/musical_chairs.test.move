
#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_musical_chairs {

  use ol_framework::musical_chairs;
  use ol_framework::mock;
  use std::vector;
  use std::fixed_point32;



  #[test(root = @ol_framework)]
  public entry fun eval_compliance_happy(root: signer) {

    let musical_chairs_default_seats = 10;
    let vals = mock::genesis_n_vals(&root, 5);
    assert!(vector::length(&vals) == 5, 7357001);

    // all vals compliant
    mock::mock_all_vals_good_performance(&root);
    let epoch = 10; // we don't evaluate epoch 0, 1
    let round = 2000; // we don't evaluate is rounds are below 1000 (one thousand)
    let (good, bad, ratio) = musical_chairs::test_eval_compliance(&root, vals,
    epoch, round);
    assert!(vector::length(&good) == 5, 7357002);
    assert!(vector::length(&bad) == 0, 7357003);
    assert!(fixed_point32::is_zero(ratio), 7357004);

    let (outgoing_compliant_set, new_set_size) =
    musical_chairs::test_stop(&root, epoch, round);

    assert!(vector::length(&outgoing_compliant_set) == 5, 7357005);
    assert!(new_set_size == (musical_chairs_default_seats + 1), 7357006);
  }

  #[test(root = @ol_framework)]
  // only one seat opens up
  public entry fun eval_compliance_one_val(root: signer) {

    let vals = mock::genesis_n_vals(&root, 5);
    assert!(vector::length(&vals) == 5, 7357001);

    // all vals compliant
    mock::mock_case_1(&root, *vector::borrow(&vals, 0));

    let epoch = 10; // we don't evaluate epoch 0, 1
    let round = 2000; // we don't evaluate is rounds are below 1000 (one thousand)
    let (good, bad, bad_ratio) = musical_chairs::test_eval_compliance(&root,
    vals, epoch, round);
    assert!(vector::length(&good) == 1, 7357002);
    assert!(vector::length(&bad) == 4, 7357003);
    assert!(!fixed_point32::is_zero(bad_ratio), 7357004);
    assert!(fixed_point32::create_from_rational(4, 5) == bad_ratio, 7357005);


    let (_outgoing_compliant_set, _new_set_size) =
    musical_chairs::test_stop(&root, epoch, round);

  }
}
