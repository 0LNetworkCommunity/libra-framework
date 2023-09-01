//! Test validator set

use diem_vm_genesis::{TestValidator, Validator};

pub fn get_test_valset(num: usize) -> Vec<Validator> {
    TestValidator::new_test_set(Some(num), None)
        .into_iter()
        .map(|v| v.data)
        .collect()
}
