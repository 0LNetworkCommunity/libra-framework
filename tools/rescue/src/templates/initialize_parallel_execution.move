script {
    use diem_framework::parallel_execution_config;
    fun main(diem_root: signer, _execute_as: signer) {
        parallel_execution_config::initialize_parallel_execution(&diem_root);
    }
}
