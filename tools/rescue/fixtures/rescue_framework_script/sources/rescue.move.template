// offchain governance (fork) script
script {
    use diem_framework::diem_governance;
    use diem_framework::stake;
    use diem_framework::block;
    use diem_framework::timestamp;
    use diem_std::debug::print;
    use std::string;

    fun main(vm_signer: signer, framework_signer: signer){
      ///////// add governance operations here /////////
      print(&string::utf8(b"plz halp"));
      ///////// end governance operations /////////

      ////// System operations needed for successful writeset //////
      // we need to advance the timestamp
      // this is because in online operation
      // multiple reconfigurations are prohibited when the
      // timestamp hasn't changed.
      let t = timestamp::now_microseconds();
      timestamp::update_global_time(&vm_signer, @0x1, (t + 1));
      // we need to touch the validtor set. Black magic.
      let _validator_set = stake::get_current_validators();
      // A reconfiguration event is necessary for a successful writeset.
      diem_governance::reconfigure(&framework_signer);
      // A dummy block event needs to be produced for a successful writeset.
      block::emit_writeset_block_event(&vm_signer, @0x1);
      ////// end writeset stuff //////

    }
}
