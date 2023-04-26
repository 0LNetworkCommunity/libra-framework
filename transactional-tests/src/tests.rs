use aptos_transactional_test_harness::run_aptos_test;

// Todo: 
datatest_stable::harness!(run_aptos_test, "tests/", r".*\.(mvir|move)$");