
script {
  // THIS IS A TEMPLATE GOVERNANCE SCRIPT
  // you can generate this file with commandline tools: `libra-framework governance --output-dir --framework-local-dir`
  use diem_framework::diem_governance;
  use std::vector;

  fun main(proposal_id: u64){
      let next_hash = vector::empty();
      let _framework_signer = diem_governance::resolve_multi_step_proposal(proposal_id, @0000000000000000000000000000000000000000000000000000000000000001, next_hash);
  }
}
