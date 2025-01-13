

module ol_framework::sortition {
    use std::vector;
    use ol_framework::randomness;
    // use diem_std::debug::print;

    /// # weighted_sample
    ///
    /// This function performs weighted random sampling without replacement. Given a vector of weights
    /// and a number `n`, it returns a vector of `n` indices, where each index is selected based on the
    /// weight of the corresponding element in the input vector.
    ///
    /// ## Parameters
    ///
    /// - `weights`: A vector of `u64` representing the weights of the items to be sampled.
    /// - `n`: A `u64` representing the number of items to sample.
    ///
    /// ## Returns
    ///
    /// A vector of `u64` containing the indices of the sampled items.
    ///
    /// ## Algorithm
    ///
    /// 1. Calculate the total weight by summing all the weights in the input vector.
    /// 2. Initialize an empty vector to store the selected indices.
    /// 3. Repeat the following steps `n` times:
    ///    - Generate a random number in the range of the total weight.
    ///    - Iterate through the weights to find the item corresponding to the random number using cumulative weights.
    ///    - Add the index of the selected item to the result vector and set its weight to 0 to remove it from the pool.
    /// 4. If the number of selected indices exceeds `n`, trim the result vector to contain exactly `n` elements.
    ///
    /// ## Example Usage
    ///
    /// ```move
    /// let weights = vector[10, 5, 15, 20, 25];
    /// let n = 3;
    /// let sampled_indices = weighted_sample(weights, n);
    /// ```
    ///
    /// This function ensures that the items are sampled based on their weights and that no item is selected more than once.

    public fun weighted_sample(weights: vector<u64>, n: u64): vector<u64> {
      let selected_indices = vector::empty();

      let i = 0;
      // sample once
      while (i < n) {
        // regenerate the weight after every selection
        let total_weight  = vector::fold(weights, 0, |acc, x| acc + x);

        // Step 1: Generate a random number in the range of total_weight
        let random_number = randomness::u64_range(0, total_weight);

        // Step 2: Find the selected item using cumulative weights
        let cumulative_weight = 0;
        let j = 0;
        while (j < vector::length(&weights)){
            let weight = *vector::borrow(&weights, j);
            cumulative_weight = cumulative_weight + weight;

            if (random_number < cumulative_weight) {
                // Select this item
                vector::push_back(&mut selected_indices, j);
                // and remove from the pool by setting its weight to 0
                // this does not shuffle the original order of the weights
                // so we can get the original indexes
                let value = vector::borrow_mut(&mut weights, j);
                *value = 0;

                break
            };
            j = j + 1;
        };

            i = i + 1;
        };

        if (vector::length(&selected_indices) > n) {
            // trim just in case
            let _ = vector::trim(&mut selected_indices, n);
        };

        return selected_indices
    }

    #[test(framework=@ol_framework)]
    fun test_weighted_sample(framework: &signer) {
        use diem_std::comparator;

        randomness::initialize_for_testing(framework);
        let weights = vector[10, 5, 15, 20, 25];

        let indexes = weighted_sample(weights, 3);
        assert!(vector::length(&indexes) == 3, 7357001);
        // TODO: check this

        let indexes_again = weighted_sample(weights, 3);
        assert!(vector::length(&indexes) == 3, 7357001);

        // should not be the same
        let res = comparator::compare(&indexes, &indexes_again);
        assert!(!comparator::is_equal(&res), 7357002);
    }



    /// Perform an in-place Fisher-Yates shuffle on a vector of indices.
    /// TL;DR take each element and swap it with a random element in the
    // paying attention not to swap an element with a previously shuffled one.
    /// # Arguments
    /// * `items` - A mutable reference to a vector of u64 indices.
    /// * `rng` - A random generator instance.
    public fun shuffle(items: &mut vector<u64>) {
      let len = vector::length(items);
      if (len == 0) { return };

      let i = 0;
      while (i < len) {
          let rand_idx = randomness::u64_range(i, len);
          vector::swap(items, rand_idx, i);
          i = i + 1;
      }
    }

    #[test(framework=@ol_framework)]
    fun test_shuffle(framework: &signer) {
        use diem_std::comparator;

        randomness::initialize_for_testing(framework);
        let original_items = vector[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let items = vector[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        shuffle(&mut items);
        assert!(vector::length(&items) == 10, 7357001);
        let res = comparator::compare(&original_items, &items);
        assert!(!comparator::is_equal(&res), 7357002);
    }
}
