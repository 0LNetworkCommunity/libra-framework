use diem_transactional_test_harness::run_diem_test;

// Todo:
datatest_stable::harness!(run_diem_test, "tests/", r".*\.(mvir|move)$");
