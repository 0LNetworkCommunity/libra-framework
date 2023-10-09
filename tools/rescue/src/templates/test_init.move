script {
    use diem_framework::block;
    fun main (vm: &signer) {
      block::emit_writeset_block_event(vm, @0x1234);
    }
}
