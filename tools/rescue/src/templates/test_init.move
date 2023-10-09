script {
    use ol_framework::ol_account;
    fun main (diem_root: signer) {
      ol_account::create_account(&diem_root, @0x1234);
    }
}
