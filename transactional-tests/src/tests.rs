use aptos_transactional_test_harness::run_aptos_test;

// Todo: fails to import aptos_transactional_test_harness, see Cargo.toml
datatest_stable::harness!(run_aptos_test, "tests", r".*\.(mvir|move)$");
// datatest_stable::harness!(run_aptos_test, "/Users/gsimsek/code/aptos-core-0L/aptos-move/aptos-transactional-test-harness/tests", r".*\.(mvir|move)$");