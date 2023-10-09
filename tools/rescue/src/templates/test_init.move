script {
    use ol_framework::slow_wallet;
    fun main (diem_root: signer) {
      slow_wallet::initialize(&diem_root);
    }
}
