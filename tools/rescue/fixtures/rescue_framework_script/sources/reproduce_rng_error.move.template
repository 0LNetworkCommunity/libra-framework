script {
    // use std::vector;
    use diem_framework::diem_governance;
    // use diem_framework::code;
    use diem_framework::stake;
    use diem_framework::block;
    use diem_framework::timestamp;
    use diem_std::debug::print;
    use ol_framework::tower_state;
    use std::string;

    fun main(vm_signer: signer, framework_signer: signer){
      print(&string::utf8(b"hi"));
      print(&string::utf8(b"toy rng works with no loop i=1"));
      // params that don't produce error
      let n = tower_state::toy_rng(3,1,10);
      print(&string::utf8(b"breaks on second loop"));
      print(&n);
      // params tower state reconfig() is using via epoch_param_reset
      let n = tower_state::toy_rng(3,2,10);
      print(&n);

      ////// standard operations needed for successful writeset //////
      let t = timestamp::now_microseconds();
      timestamp::update_global_time(&vm_signer, @0x1, (t + 1));
      let _validator_set = stake::get_current_validators();
      diem_governance::reconfigure(&framework_signer);
      block::emit_writeset_block_event(&vm_signer, @0x1);
      ////// end writeset stuff //////

    }
}
