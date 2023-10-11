script {
    use diem_framework::transaction_publishing_option;
    fun main(diem_root: signer) {
        transaction_publishing_option::halt_all_transactions(&diem_root);
    }
}
