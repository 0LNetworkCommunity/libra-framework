script {
    // use std::vector;
    use diem_framework::diem_governance;
    // use diem_framework::code;
    use diem_framework::stake;
    use diem_framework::block;
    // use diem_framework::version;
    use diem_std::debug::print;

    fun main(vm_signer: signer, framework_signer: signer){
      print(&@0x1);
      let _validator_set = stake::get_current_validators();
      diem_governance::reconfigure(&framework_signer);
      block::emit_writeset_block_event(&vm_signer, @0x1);

    }
}
