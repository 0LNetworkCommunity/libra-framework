module ol_framework::migrations {
  use std::vector;
  use std::string;
  use std::error;
  use diem_framework::system_addresses;
  use ol_framework::epoch_helper;

  use diem_std::debug::print;

  // migrations
  use ol_framework::vouch_migration;

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

  public fun execute(root: &signer) acquires Migrations {
    // ensure ol_framework
    system_addresses::assert_ol(root);

    // execute all migrations
    execute_vouch_migration(root, 1);
  }

  // TODO: confirm that it is not possible the usage of higher order function
  // to avoid code duplication
  fun execute_vouch_migration(root: &signer, mig_number: u64) acquires Migrations {
    if (can_execute_migration(mig_number)) {

      // migrate vouches
      vouch_migration::migrate_my_vouches();

      // update last_migration
      register_migration(root, mig_number, b"Vouch migration initializes GivenVouches, ReceivedVouches, and drop MyVouches");
    };
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

    print(&string::utf8(b">>> Migration executed:"));
    print(&mig_number);
    print(&string::utf8(description));
  }

  public fun get_last_migration_number(): u64 acquires Migrations {
    if (!exists<Migrations>(@ol_framework)) {
      return 0
    };

    let state = borrow_global<Migrations>(@ol_framework);
    state.last_migration
  }

  public fun get_last_migrations_history(): (u64, u64, vector<u8>) acquires Migrations {
    assert!(exists<Migrations>(@ol_framework), error::invalid_state(EMIGRATIONS_NOT_INITIALIZED));

    let state = borrow_global<Migrations>(@ol_framework);
    let last_migration = vector::borrow(&state.history, vector::length(&state.history) -1);
    (last_migration.number, last_migration.epoch, last_migration.description)
  }

}
