module ol_framework::migrations {
  use std::vector;
  use std::string;
  use std::error;
  use diem_framework::system_addresses;
  use ol_framework::donor_voice_migration;
  use ol_framework::epoch_helper;
  use ol_framework::root_of_trust;
  use ol_framework::migration_capability::MigrationCapability;

  use diem_std::debug::print;

  //////// CONST ////////
  const EMIGRATIONS_NOT_INITIALIZED: u64 = 1;

  //////// STRUCTS ////////
  struct Migration has store, copy {
    number: u64,
    epoch: u64,
    description: vector<u8>,
  }

  struct Migrations has key {
    last_migration: u64,
    history: vector<Migration>,
  }

  public fun execute(root: &signer, migration_cap: &MigrationCapability) acquires Migrations {
    // ensure ol_framework
    system_addresses::assert_ol(root);

    // execute all migrations
    if (apply_migration(root, 1, b"Vouch migration initializes GivenVouches, ReceivedVouches, and drop MyVouches")) {
      // Should have concluded in V7
      // vouch_migration::migrate_my_vouches();
    };

    if (apply_migration(root, 2, b"If root of trust is not initialize use 2021 genesis set")) {
      root_of_trust::genesis_initialize(root, root_of_trust::genesis_root());
    };

    if (apply_migration(root, 3, b"All community endowments need new data structures")) {
      donor_voice_migration::v8_state_migration(root, migration_cap);
    };

  }

  fun apply_migration(root: &signer, mig_number: u64, description: vector<u8>): bool acquires Migrations {
    if (can_execute_migration(mig_number)) {
      print(&string::utf8(b">>> Migration started:"));
      print(&mig_number);
      print(&string::utf8(description));

      register_migration(root, mig_number, description);
      true
    } else {
      false
    }
  }

  fun can_execute_migration(mig_number: u64): bool acquires Migrations {
    get_last_migration_number() < mig_number
  }

  fun register_migration(root: &signer, mig_number: u64, description: vector<u8>) acquires Migrations {
    let epoch = epoch_helper::get_current_epoch();

    if (exists<Migrations>(@ol_framework)) {
      // update
      let state = borrow_global_mut<Migrations>(@diem_framework);
      state.last_migration = mig_number;
      vector::push_back(&mut state.history, Migration {
        number: mig_number,
        epoch: epoch,
        description: description,
      });
    } else {
      // initialize
      move_to<Migrations>(root, Migrations {
        last_migration: mig_number,
        history: vector[Migration {
          number: mig_number,
          epoch: epoch,
          description: description,
        }],
      });
    };
  }

  #[view]
  /// see which migration ran most recently
  public fun get_last_migration_number(): u64 acquires Migrations {
    if (!exists<Migrations>(@ol_framework)) {
      return 0
    };

    let state = borrow_global<Migrations>(@ol_framework);
    state.last_migration
  }

  #[view]
  /// get the state of the migrations
  public fun get_last_migrations_history(): (u64, u64, vector<u8>) acquires Migrations {
    assert!(exists<Migrations>(@ol_framework), error::invalid_state(EMIGRATIONS_NOT_INITIALIZED));

    let state = borrow_global<Migrations>(@ol_framework);
    let last_migration = vector::borrow(&state.history, vector::length(&state.history) -1);
    (last_migration.number, last_migration.epoch, last_migration.description)
  }

  /// function to search through history to see  if a migration number has already been executed
  public fun has_migration_executed(mig_number: u64): bool acquires Migrations {
    assert!(exists<Migrations>(@ol_framework), error::invalid_state(EMIGRATIONS_NOT_INITIALIZED));

    let state = borrow_global<Migrations>(@ol_framework);
    let history_len = vector::length(&state.history);
    let i = 0;
    while (i < history_len) {
      let migration = vector::borrow(&state.history, i);
      if (migration.number == mig_number) {
        return true
      };
      i = i + 1;
    };
    false
  }


}
