/// This module provides utility functions for handling and manipulating vectors of
/// addresses and their corresponding values.

module ol_framework::address_utils {
  use std::error;
  use std::vector;
  use diem_framework::randomness;

  // Error code for different length of addresses and values
  const EDIFFERENT_LENGTH: u64 = 1;

  // A, B, C, easy as 1, 2, 3

  // Bubble sort addresses and corresponding values
  public fun sort_by_values(addresses: &mut vector<address>, values: &mut vector<u64>) {
    assert!(vector::length(addresses) == vector::length(values), error::invalid_argument(EDIFFERENT_LENGTH));
    let len = vector::length<u64>(values);
    let i = 0;
    while (i < len) {
      let j = 0;
      while (j < len - i - 1) {
        let value_j = *vector::borrow<u64>(values, j);
        let value_jp1 = *vector::borrow<u64>(values, j + 1);
        if (value_j > value_jp1) {
          vector::swap<u64>(values, j, j + 1);
          vector::swap<address>(addresses, j, j + 1);
        };
        j = j + 1;
      };
      i = i + 1;
    };
  }

  // Shuffle addresses with the same values to ensure randomness position
  public fun shuffle_duplicates(addresses: &mut vector<address>, values: &vector<u64>) {
    // belt and suspenders, if migration didn't happen.
    // assert!(randomness::is_init(), error::invalid_state(ERANDOM_INIT_ERROR));

    assert!(vector::length(addresses) == vector::length(values), error::invalid_argument(EDIFFERENT_LENGTH));
    let len = vector::length(values);
    let i = 0;
    while (i < len) {
      let j = i + 1;
      while (j < len && *vector::borrow(values, i) == *vector::borrow(values, j)) {
        j = j + 1;
      };
      if (j > i + 1) {
        let slice_len = j - i;
        let perm = randomness::permutation(slice_len);
        let temp_addresses = vector::empty<address>();
        let k = 0;
        while (k < slice_len) {
          let pos = i + k;
          vector::push_back(&mut temp_addresses, *vector::borrow(addresses, pos));
          k = k + 1;
        };
        k = 0;
        while (k < slice_len) {
          let perm_pos = *vector::borrow(&perm, k);
          *vector::borrow_mut(addresses, i + k) = *vector::borrow(&temp_addresses, perm_pos);
          k = k + 1;
        };
      };
      i = j;
    };
  }

  // Bubble Sort tests

  #[test]
  #[expected_failure(abort_code = 0x10001, location = ol_framework::address_utils)]
  fun test_sort_by_values_different_lengths() {
    let values = vector[1, 2, 3];
    let addresses = vector[@0x1, @0x2];
    // This should trigger an assertion error due to different lengths
    sort_by_values(&mut addresses, &mut values);
  }

  #[test]
  fun test_sort_empty_vectors() {
    let values: vector<u64> = vector::empty();
    let addresses: vector<address> = vector::empty();
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[], 10002);
    assert!(addresses == vector[], 10003);
  }

  #[test]
  fun test_sort_single_element() {
    let values = vector[10];
    let addresses = vector[@0x1];
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[10], 10004);
    assert!(addresses == vector[@0x1], 10005);
  }

  #[test]
  fun test_sort_already_sorted() {
    let values = vector[1, 2, 3, 4, 5];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[1, 2, 3, 4, 5], 10006);
    assert!(addresses == vector[@0x1, @0x2, @0x3, @0x4, @0x5], 10007);
  }

  #[test]
  fun test_sort_reverse_order() {
    let values = vector[5, 4, 3, 2, 1];
    let addresses = vector[@0x5, @0x4, @0x3, @0x2, @0x1];
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[1, 2, 3, 4, 5], 10008);
    assert!(addresses == vector[@0x1, @0x2, @0x3, @0x4, @0x5], 10009);
  }

  #[test]
  fun test_sort_with_duplicates() {
    let values = vector[4, 2, 2, 3, 1];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[1, 2, 2, 3, 4], 10010);
    assert!(addresses == vector[@0x5, @0x2, @0x3, @0x4, @0x1], 10011);
  }

  #[test]
  fun test_sort_random_order() {
    let values = vector[3, 1, 4, 5, 2];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[1, 2, 3, 4, 5], 10012);
    assert!(addresses == vector[@0x2, @0x5, @0x1, @0x3, @0x4], 10013);
  }

  #[test]
  fun test_sort_all_elements_equal() {
    let values = vector[3, 3, 3, 3, 3];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    sort_by_values(&mut addresses, &mut values);
    assert!(values == vector[3, 3, 3, 3, 3], 10014);
    assert!(addresses == vector[@0x1, @0x2, @0x3, @0x4, @0x5], 10015);
  }


  // Shuffle Tests

  #[test]
  #[expected_failure(abort_code = 0x10001, location = ol_framework::address_utils)]
  fun test_shuffle_duplicates_different_lengths() {
    let values = vector[1, 2, 3];
    let addresses = vector[@0x1, @0x2];
    // This should trigger an assertion error due to different lengths
    shuffle_duplicates(&mut addresses, &mut values);
  }

  #[test]
  fun test_shuffle_no_duplicates() {
    // No duplicates in the values vector
    let values = vector[1, 2, 3, 4, 5];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    shuffle_duplicates(&mut addresses, &mut values);
    assert!(values == vector[1, 2, 3, 4, 5], 10017);
    assert!(addresses == vector[@0x1, @0x2, @0x3, @0x4, @0x5], 10018);
  }

  #[test(root = @ol_framework)]
  fun test_shuffle_with_duplicates(root: &signer) {
    // One group of duplicates in the values vector
    randomness::initialize_for_testing(root);
    let values = vector[1, 2, 2, 3, 4];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    let original_addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5];
    let shuffled = false;
    let i = 0;

    while (i < 10) {
      shuffle_duplicates(&mut addresses, &mut values);
      if (addresses != original_addresses) {
        shuffled = true;
        break
      };
      i = i + 1;
    };

    assert!(values == vector[1, 2, 2, 3, 4], 10019);
    assert!(shuffled, 10020);
  }

  #[test(root = @ol_framework)]
  fun test_shuffle_multiple_duplicate_groups(root: &signer) {
    // Multiple groups of duplicates in the values vector
    randomness::initialize_for_testing(root);
    let values = vector[1, 2, 2, 3, 3, 4];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5, @0x6];
    let original_addresses = vector[@0x1, @0x2, @0x3, @0x4, @0x5, @0x6];
    let shuffled = false;
    let i = 0;

    while (i < 10) {
      shuffle_duplicates(&mut addresses, &mut values);
      if (addresses != original_addresses) {
        shuffled = true;
        break
      };
      i = i + 1;
    };

    assert!(values == vector[1, 2, 2, 3, 3, 4], 10021);
    assert!(shuffled, 10022);
  }

  #[test(root = @ol_framework)]
  fun test_shuffle_all_elements_equal(root: &signer) {
    // All elements in the values vector are the same
    randomness::initialize_for_testing(root);
    let values = vector[2, 2, 2, 2];
    let addresses = vector[@0x1, @0x2, @0x3, @0x4];
    let original_addresses = vector[@0x1, @0x2, @0x3, @0x4];
    let shuffled = false;
    let i = 0;

    while (i < 10) {
      shuffle_duplicates(&mut addresses, &mut values);
      if (addresses != original_addresses) {
        shuffled = true;
        break
      };
      i = i + 1;
    };

    assert!(values == vector[2, 2, 2, 2], 10023);
    assert!(shuffled, 10024);
  }
}
