module ol_framework::vector_helper {
  use std::vector::{length, borrow};

  public fun compare<Element>(a: &vector<Element>, b: &vector<Element>): bool {
      let i = 0;
      let len_a = length(a);
      let len_b = length(b);
      if (len_a != len_b) { return false };
      while (i < len_a) {
          let num_a = borrow(a, i);
          let num_b = borrow(b, i);
          if (num_a == num_b) {
              i = i + 1;  
          } else {
              return false
          }
      };
      true
  }

}