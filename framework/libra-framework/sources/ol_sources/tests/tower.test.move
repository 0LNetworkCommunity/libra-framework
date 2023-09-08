#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_tower {
  use ol_framework::mock;
  use ol_framework::tower_state;
  use ol_framework::vdf_fixtures;
  // use ol_framework::testnet;

  // use std::debug::print;

  #[test(root = @ol_framework)]
  fun epoch_changes_difficulty(root: signer) {
    mock::genesis_n_vals(&root, 4);
    mock::ol_initialize_coin(&root);

    mock::tower_default(&root); // make all the validators initialize towers
    // because we need randomness for the toy rng

    let (diff, sec) = tower_state::get_difficulty();

    // check the state started with the testnet defaults
    assert!(diff==100, 735701);
    assert!(sec==512, 735702);

    mock::trigger_epoch(&root);

    let (diff, _sec) = tower_state::get_difficulty();
    assert!(diff!=100, 735703);
  }

  #[test(root = @ol_framework, alice = @0x87515d94a244235a1433d7117bc0cb154c613c2f4b1e67ca8d98a542ee3f59f5)]
  fun init_tower_state(root: signer, alice: signer){
    mock::ol_test_genesis(&root);
    mock::ol_initialize_coin(&root);

    tower_state::init_miner_state(
          &alice,
          &vdf_fixtures::weso_alice_0_easy_chal(),
          &vdf_fixtures::weso_alice_0_easy_sol(),
          vdf_fixtures::easy_difficulty(),
          350, // wesolowski
      );

    }


  #[test(root = @ol_framework)]
  fun toy_rng_state(root: signer) {
    mock::genesis_n_vals(&root, 4);
    mock::ol_initialize_coin(&root);

    mock::tower_default(&root); // make all the validators initialize towers
    // because we need randomness for the toy rng

    let num = tower_state::toy_rng(1, 3, 0);

    assert!(num == 184, 7357001);

  }

}