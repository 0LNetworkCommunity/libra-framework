#[test_only]
/// tests for external apis, and where a dependency cycle with genesis is created.
module ol_framework::test_tower {
  use ol_framework::mock;
  use std::debug::print;
  use ol_framework::tower_state;
  use ol_framework::vdf_fixtures;


  #[test(root = @ol_framework)]
  fun epoch_changes_difficulty(root: signer) {
    mock::ol_test_genesis(&root);
    mock::ol_initialize_coin(&root);

    let (diff, sec) = tower_state::get_difficulty();
    print(&diff);
    print(&sec);

    // check the state started with the testnet defaults
    assert!(diff==100, 735701);
    assert!(sec==512, 735702);

    mock::trigger_epoch(&root);

    let (diff, sec) = tower_state::get_difficulty();
    print(&diff);
    print(&sec);
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

      // // print(&TowerState::get_epochs_compliant(@Alice));
      // assert!(TowerState::get_tower_height(@Alice) == 0, 735701);
      // assert!(TowerState::get_epochs_compliant(@Alice) == 0, 735702);
      // assert!(TowerState::get_count_in_epoch(@Alice) == 1, 735703);
      // // print(&TowerState::get_miner_list());
      // assert!(Vector::length<address>(&TowerState::get_miner_list()) == 2, 735704); 
          // includes the dummy validator from genesis
    }

}