#[test_only]
module ol_framework::test_validator_vouch {
  use std::signer;
  use std::vector;
  use ol_framework::vouch;
  use ol_framework::vouch_txs;
  use ol_framework::mock;
  use ol_framework::proof_of_fee;

  // Happy Day scenarios
  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun vouch_for_unrelated(root: &signer, alice: &signer, carol: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 3, false);
    vouch::set_vouch_price(root, 0);

    // check that no vouches exist
    vector::for_each(vector[@0x1000a, @0x1000b, @0x1000c], |addr| {
      let (given_vouches, given_epochs) = vouch::get_given_vouches(addr);
      assert!(given_vouches == vector::empty(), 73570001);
      assert!(given_epochs == vector::empty(), 73570002);
      let (received_vouches, received_epochs) = vouch::get_received_vouches(addr);
      assert!(received_vouches == vector::empty(), 73570003);
      assert!(received_epochs == vector::empty(), 73570004);
    });

    // alice vouches for bob
    vouch::vouch_for(alice, @0x1000b);

    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b], 73570005);
    assert!(given_epochs == vector[0], 73570006);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a], 73570007);
    assert!(received_epochs == vector[0], 73570008);

    // fast forward to epoch 1
    mock::trigger_epoch(root);

    // carol vouches for bob
    vouch::vouch_for(carol, @0x1000b);

    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b], 73570005);
    assert!(given_epochs == vector[0], 73570006);

    // check carol
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000c);
    assert!(given_vouches == vector[@0x1000b], 73570009);
    assert!(given_epochs == vector[1], 73570010);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a, @0x1000c], 73570011);
    assert!(received_epochs == vector[0, 1], 73570012);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun revoke_vouch(root: &signer, alice: &signer, carol: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 3, false);
    vouch::set_vouch_price(root, 0);

    // alice and carol vouches for bob
    vouch::vouch_for(alice, @0x1000b);
    vouch::vouch_for(carol, @0x1000b);

    // alice revokes vouch for bob
    vouch::revoke(alice, @0x1000b);

    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector::empty(), 73570013);
    assert!(given_epochs == vector::empty(), 73570014);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000c], 73570015);
    assert!(received_epochs == vector[0], 73570016);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b, carol = @0x1000c)]
  fun epoch_boundary_update_vouch_price(root: &signer) {
    // check default vouch micro-price
    assert!(vouch::get_vouch_price() == 1_000, 73570025);

    // initialize vals and vouch price
    mock::genesis_n_vals(root, 3);
    vouch::set_vouch_price(root, 0);
    assert!(vouch::get_vouch_price() == 0, 73570025);

    // mock to increase reward on the next epoch
    proof_of_fee::test_mock_reward(root, 5_000, 1_000, 10, vector[10, 10, 10, 10, 10, 10]);
    let (reward, _, _, _ ) = proof_of_fee::get_consensus_reward();
    assert!(reward == 5_000, 73570026);

    mock::trigger_epoch(root);

    // check new vouch price
    let (reward, _, _, _ ) = proof_of_fee::get_consensus_reward();
    assert!(reward == 5_250, 73570027);
    assert!(vouch::get_vouch_price() == 5_250, 73570026);

    mock::trigger_epoch(root);

    // check new vouch price
    let (reward, _, _, _ ) = proof_of_fee::get_consensus_reward();
    assert!(reward == 5_512, 73570027);
    assert!(vouch::get_vouch_price() == 5_512, 73570026);

    // mock to descrease reward on the next epoch
    proof_of_fee::test_mock_reward(root, 10_000, 9_600, 960, vector[960, 960, 960, 960, 960, 960]);

    mock::trigger_epoch(root);

    // check new vouch price
    let (reward, _, _, _ ) = proof_of_fee::get_consensus_reward();
    assert!(reward == 9_500, 73570026);
    assert!(vouch::get_vouch_price() == 9_500, 73570027);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  fun update_vouch(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 2, false);
    vouch::set_vouch_price(root, 0);

    print(&11000);
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    print(&given_vouches);
    print(&given_epochs);
    // alice vouches for bob
    vouch::vouch_for(alice, @0x1000b);

    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b], 73570005);
    assert!(given_epochs == vector[0], 73570006);
    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a], 73570007);
    assert!(received_epochs == vector[0], 73570008);
    print(&11003);

    // fast forward to epoch 1
    mock::trigger_epoch(root);
    print(&11004);

    // alice vouches for bob again
    vouch::vouch_for(alice, @0x1000b);
    // check alice
    let (given_vouches, given_epochs) = vouch::get_given_vouches(@0x1000a);
    assert!(given_vouches == vector[@0x1000b], 73570005);
    assert!(given_epochs == vector[1], 73570006);
    print(&11006);

    // check bob
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000b);
    assert!(received_vouches == vector[@0x1000a], 73570007);
    assert!(received_epochs == vector[1], 73570008);
  }

  // Sad Day scenarios

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x10001, location = ol_framework::vouch)]
  fun vouch_for_self(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 1, false);
    vouch::set_vouch_price(root, 0);

    // alice vouches for herself
    vouch::vouch_for(alice, @0x1000a);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x10001, location = ol_framework::vouch)]
  fun revoke_self_vouch(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 1, false);

    // alice try to revoke herself
    vouch::revoke(alice, @0x1000a);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x10005, location = ol_framework::vouch)]
  fun revoke_not_vouched(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 2, false);

    // alice vouches for bob
    vouch::revoke(alice, @0x1000b);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 196618, location = ol_framework::vouch_limits)]
  fun vouch_over_max_received(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::genesis_n_vals(root, 2);
    // mock::create_validator_accounts(root, 2, true);
    vouch::set_vouch_price(root, 0);

    let users = mock::create_test_end_users(root, 10, 0);

    let i = 0;
    while (i < 10) {
      let sig = vector::borrow(&users, i);
      // init vouch for 10 validators
      vouch::init(sig);
      // alice vouches for 10 validators
      vouch_txs::vouch_for(alice, signer::address_of(sig));
      i = i + 1;
    };

    // should fail
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x30006, location = ol_framework::ol_account)]
  fun vouch_without_coins(root: &signer, alice: &signer) {
    // create vals without vouches
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 2, false);
    vouch::set_vouch_price(root, 9_999);

    // alice vouches for bob without coins
    vouch_txs::vouch_for(alice, @0x1000b);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x30006, location = ol_framework::vouch)]
  fun vouch_without_init(alice: &signer) {
    // alice vouches for bob without init
    vouch::vouch_for(alice, @0x1000b);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x30002, location = ol_framework::vouch)]
  fun vouch_for_account_not_init(root: &signer, alice: &signer) {
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 1, false);

    // alice vouches for bob without init
    vouch::vouch_for(alice, @0x1000b);
  }

  #[test(root = @ol_framework, alice = @0x1000a, bob = @0x1000b)]
  #[expected_failure(abort_code = 0x30006, location = ol_framework::vouch)]
  fun revoke_without_init(alice: &signer) {
    // alice try to revoke bob without vouch
    vouch::revoke(alice, @0x1000b);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x10005, location = ol_framework::vouch)]
  fun revoke_account_not_vouched(root: &signer, alice: &signer) {
    mock::ol_test_genesis(root);
    mock::create_validator_accounts(root, 2, false);

    // alice try to revoke bob without vouch
    vouch::revoke(alice, @0x1000b);
  }


  #[test(root = @ol_framework, alice = @0x1000a)]
  fun true_friends_not_init() {
    // alice try to get true friends in list without init
    let result = vouch::true_friends(@0x1000a);

    assert!(result == vector::empty(), 73570028);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  fun true_friends_in_list_not_init() {
    // alice try to get true friends in list without init
    let (list, size) = vouch::true_friends_in_list(@0x1000a, &vector::singleton(@0x1000b));

    assert!(list == vector::empty(), 73570028);
    assert!(size == 0, 73570029);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  fun get_received_vouches_not_init() {
    // alice try to get received vouches without init
    let (received_vouches, received_epochs) = vouch::get_received_vouches(@0x1000a);

    assert!(received_vouches == vector::empty(), 73570030);
    assert!(received_epochs == vector::empty(), 73570031);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  #[expected_failure(abort_code = 0x30003, location = ol_framework::vouch)]
  fun get_given_vouches_not_init() {
    // alice try to get given vouches without init
    vouch::get_given_vouches(@0x1000a);
  }

  #[test(root = @ol_framework, alice = @0x1000a)]
  fun get_all_vouchers_not_init() {
    // alice try to get all vouchers without init
    let all_vouchers = vouch::all_vouchers(@0x1000a);

    assert!(all_vouchers == vector::empty(), 73570034);
  }

}
