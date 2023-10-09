script {
    use diem_framework::reconfiguration;
    fun main (diem_root: signer) {
      reconfiguration::reconfigure_for_rescue(&diem_root);
    }
}
